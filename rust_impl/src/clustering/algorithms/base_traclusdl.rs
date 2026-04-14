use std::sync::atomic::Ordering;

use eframe::egui::debug_text::print;

use super::super::geometry::{segment::Segment, trajectory::Trajectory};
use super::super::objects::{
    cluster::Cluster,
    cluster_member::{ClusterMember, ClusterSeed},
};
use super::super::storage::{
    clustered_trajectories::ClusteredTrajectories, raw_trajectories::RawTrajectories,
};
use crate::io::args::TraclusArgs;
use crate::utils::events::app_events::{AppEvent, ComputationType};
use crate::utils::events::event_singleton::emit;
use crate::utils::gui_parallel_runner::StopFlag;

pub const TICK_EVERY: usize = 25; // how many trajectories between progress events

/// Base trait for TraClus algorithm implementations.
///
/// This trait defines the contract that all TraClus variants must follow,
/// providing both required methods and overridable default implementations
/// for the core clustering algorithm.
pub trait TraclusAlgorithm {
    // ============================================================
    // Shared Data Accessors
    // ============================================================
    fn args(&self) -> &TraclusArgs;

    fn stop_flag(&self) -> &Option<StopFlag>;
    fn set_stop_flag(&mut self, stop_flag: StopFlag);

    // ============================================================
    // Required Methods (Must Be Implemented by Implementations)
    // ============================================================

    /// Performs DB-SCAN clustering on trajectory segments.
    ///
    /// This is the main method to partitions the trajectory into clusters
    /// based constraints and then creates the appropriate corridors.
    /// # Arguments
    /// * `raw_trajectories` - The raw trajectory storage containing all trajectories
    /// * `clustered_trajectories` - The clustered trajectory storage to populate with clusters
    /// * `emitter` - The event emitter for sending computation events
    /// # Returns
    /// * `true` if clustering completed successfully
    /// * `false` if clustering was stopped early due to a stop signal
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) -> bool;

    // ============================================================
    // Default Methods (Can Be Overridden If Needed)
    // ============================================================

    /// Finds all reachable trajectory segments from a given seed segment.
    ///
    /// This method applies four constraints to determine reachability:
    /// 1. **Same trajectory constraint**: Excludes segments from the same trajectory
    /// 2. **Angle constraint**: Filters by direction similarity (max_angle)
    /// 3. **Distance constraint**: Filters by spatial proximity (max_dist)
    /// 4. **Density constraint**: Ensures minimum cluster weight (min_density)
    ///
    /// # Time Complexity
    /// O(n × d / bucket_size) where n is nearby trajectories, d is avg trajectory length
    ///
    /// # Arguments
    /// * `seed` - The seed segment to cluster around
    /// * `nearby_trajs` - Candidate trajectories within spatial proximity
    ///
    /// # Returns
    /// * `Some(Cluster)` if density constraint is met
    /// * `None` if the cluster doesn't meet minimum density requirements
    fn cluster_reachable_segs(
        &self,
        seed: ClusterSeed,
        nearby_trajs: &Vec<&Trajectory>,
    ) -> Option<Cluster> {
        let mut cluster: Cluster = Cluster::new(seed, Vec::new());
        let seed_ref: &ClusterSeed = &cluster.seed;
        let mut local_weight: u32 = seed_ref.cm.weight;

        for nearby_traj in nearby_trajs {
            // Constraint 1: Skip if same trajectory
            if seed_ref.cm.traj_id == nearby_traj.id {
                continue;
            }

            // Constraint 2: Check angle difference
            let angle_diff: f64 = (seed_ref.angle - nearby_traj.angle).abs();
            let min_angle_diff: f64 = angle_diff.min(360.0 - angle_diff);
            if min_angle_diff > self.args().max_angle + 1e-9 {
                continue;
            }

            // Constraint 3: Check spatial distance
            let (dist, segment_id) = nearby_traj.distance_to_point(&seed_ref.cm.center);
            if dist > self.args().max_dist + 1e-9 {
                continue;
            }

            // emit(AppEvent::PrintInfo {
            //     messages: vec![format!(
            //         "Checking distance from seed to trajectory {}, {}: ",
            //         nearby_traj.id, segment_id
            //     )],
            // });
            // Add qualifying segment as a candidate
            let segment: &Segment = nearby_traj.segment(segment_id).unwrap();
            let candidate: ClusterMember = ClusterMember::new(
                nearby_traj.id,
                segment_id,
                nearby_traj.weight,
                segment.middle.clone(),
                segment.start.clone(),
            );
            local_weight += candidate.weight;
            cluster.candidates.push(candidate);
        }

        // Constraint 4: Check density threshold (including seed weight)
        if local_weight < self.args().min_density {
            return None;
        }

        Some(cluster)
    }

    /// Expands a cluster by iteratively processing candidate segments.
    ///
    /// This method implements a breadth-first expansion where each candidate
    /// segment is used as a new seed to find additional reachable segments.
    /// The process continues until no new candidates are found.
    ///
    /// # Time Complexity
    /// O(m' × cluster_reachable_segs) = O(m' × n × d / bucket_size)
    /// where m' is the number of members in the final cluster
    ///
    /// # Arguments
    /// * `cluster` - The cluster to expand (modified in place)
    /// * `nearby_trajs` - Candidate trajectories to consider
    ///
    /// # Returns
    /// A mutable reference to the expanded cluster
    fn expand_segment_cluster<'a>(
        &self,
        cluster: &'a mut Cluster,
        nearby_trajs: &Vec<&Trajectory>,
    ) -> &'a mut Cluster {
        while !cluster.candidates.is_empty() {
            let mut new_clusters: Vec<Cluster> = Vec::new();

            // Process candidates in reverse order for consistency with v1 behavior
            for candidate in cluster.candidates.iter().rev() {
                let seed_member: ClusterSeed = ClusterSeed::new(
                    ClusterMember::new_from_candidate(candidate),
                    cluster.seed.angle,
                );

                if let Some(new_cluster) = self.cluster_reachable_segs(seed_member, nearby_trajs) {
                    new_clusters.push(new_cluster);
                }
            }

            // Promote all candidates to members
            cluster.move_candidates_to_members();

            // Merge newly discovered clusters
            for new_cluster in new_clusters {
                cluster.merge_clusters(new_cluster);
            }
        }

        cluster
    }

    /// Initializes a cluster from a seed segment.
    ///
    /// This is a convenience method that finds the initial reachable segments
    /// for a given seed without performing expansion.
    ///
    /// # Time Complexity
    /// O(n × d / bucket_size)
    ///
    /// # Arguments
    /// * `seed` - Tuple of (segment, trajectory) to use as the initial seed
    /// * `nearby_trajs` - Candidate trajectories to consider
    ///
    /// # Returns
    /// * `Some(Cluster)` if initial clustering succeeds
    /// * `None` if no valid cluster can be formed
    fn initial_segment_cluster(
        &self,
        seed: (&Segment, &Trajectory),
        nearby_trajs: &Vec<&Trajectory>,
    ) -> Option<Cluster> {
        let member: ClusterMember = ClusterMember::new_from_traj(seed.1, seed.0);
        let seed_member: ClusterSeed = ClusterSeed::new(member, seed.1.angle);
        self.cluster_reachable_segs(seed_member, nearby_trajs)
    }

    /// Serially cycle through all trajectories and fill non-clustered segments
    ///
    /// # Arguments
    /// * `raw_trajectories` - The raw trajectory storage containing all trajectories
    /// * `clustered_trajectories` - The clustered trajectory storage to populate with clusters
    fn fill_non_clustered_segments(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) {
        for bucket in &raw_trajectories.traj_buckets {
            for traj_seed in &bucket.trajectories {
                clustered_trajectories.fill_non_clustered_segments(traj_seed);
            }
        }
    }

    // ============================================================
    // Emitter Helpers (For Emitting Progress Events During Clustering)
    // ============================================================

    /// Returns true if a stop has been requested.
    fn is_stopped(&self) -> bool {
        if let Some(stop_flag) = self.stop_flag() {
            return stop_flag.load(Ordering::Relaxed);
        }
        false
    }

    fn tick_clustering(&self, count: &mut usize) {
        if *count % TICK_EVERY == 0 {
            emit(AppEvent::ComputationProgress {
                computation_type: ComputationType::Clustering,
                increment_progress: TICK_EVERY,
            });
        }
        *count += 1;
    }

    fn tick_remove_duplicates(&self, num_last_elements: usize, num_current_elements: usize) {
        emit(AppEvent::ComputationProgress {
            computation_type: ComputationType::RemoveDuplicates,
            increment_progress: num_last_elements - num_current_elements,
        });
    }

    fn emit_start_clustering(&self, raw_trajectories: &RawTrajectories) {
        emit(AppEvent::ComputationStart {
            computation_type: ComputationType::Clustering,
            max_progress: raw_trajectories.get_total_trajectories(),
        });
    }

    fn emit_complete_clustering(&self) {
        emit(AppEvent::ComputationComplete {
            computation_type: ComputationType::Clustering,
        });
    }

    fn emit_start_remove_duplicates(&self, clustered_trajectories: &ClusteredTrajectories) {
        emit(AppEvent::ComputationStart {
            computation_type: ComputationType::RemoveDuplicates,
            max_progress: clustered_trajectories.get_size_priority_queue(),
        });
    }

    fn emit_complete_remove_duplicates(&self) {
        emit(AppEvent::ComputationComplete {
            computation_type: ComputationType::RemoveDuplicates,
        });
    }
}
