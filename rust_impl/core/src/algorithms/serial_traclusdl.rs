use super::super::geometry::trajectory::Trajectory;
use super::super::objects::cluster::Cluster;
use super::super::objects::corridor::Corridor;
use super::super::storage::{
    clustered_trajectories::ClusteredTrajectories, raw_trajectories::RawTrajectories,
};
use super::base_traclusdl::TraclusAlgorithm;

use crate::io::args::TraclusArgs;
use crate::utils::events::event_singleton::emit_timed_perf;
use crate::utils::gui_parallel_runner::StopFlag;

pub struct SerialTraclusDL {
    args: TraclusArgs,
    stop_flag: Option<StopFlag>,
}

impl SerialTraclusDL {
    pub fn new(args: TraclusArgs) -> Self {
        Self {
            args,
            stop_flag: None,
        }
    }

    /// Completes the serial clustering process by iterating over angle buckets
    /// Each trajectory inside each bucket is computed
    ///
    /// # Arguments
    /// * `raw_trajectories` - The raw trajectory storage containing all trajectories
    /// * `clustered_trajectories` - The clustered trajectory storage to populate with clusters
    fn complete_serial_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) {
        for bucket in &raw_trajectories.traj_buckets {
            // Get a copy of nearby trajectories for this angle bucket
            emit_timed_perf("Copy_Nearby_Trajectories", true, None);
            let nearby_trajs: Vec<Trajectory> =
                raw_trajectories.vec_nearby_angle(bucket.angle_start);
            emit_timed_perf("Copy_Nearby_Trajectories", false, None);

            // Iterate over trajectories in this bucket serially
            for traj_seed in &bucket.trajectories {
                let clusters: Vec<Cluster> =
                    self.individual_trajectory_clustering(traj_seed, &nearby_trajs);
                clustered_trajectories.add_list_cluster(clusters);

                // Tick after every trajectory (count = 1)
                self.tick_clustering(1);

                // Check for stop signal to bail out early
                if self.is_stopped() {
                    return;
                }
            }
        }
    }

    /// Clusters an individual trajectory against nearby trajectories.
    /// For each segment:
    /// - Attempts to create an initial cluster if density requirements are met
    /// - Expands the cluster to include all reachable segments
    /// - Stores the completed cluster
    ///
    /// # Arguments
    /// * `traj_seed` - The trajectory to use as a clustering seed
    /// * `nearby_trajs` - Vector of nearby trajectories to consider for clustering
    /// # Returns
    /// * A vector of clusters formed from the trajectory segments
    fn individual_trajectory_clustering(
        &self,
        traj_seed: &Trajectory,
        nearby_trajs: &[Trajectory],
    ) -> Vec<Cluster> {
        let mut cluster_group: Vec<Cluster> = Vec::new();

        for seed_segment in traj_seed.segments_iter() {
            // Try to form an initial cluster from this seed segment
            let cluster: Option<Cluster> =
                self.initial_segment_cluster((&seed_segment, &traj_seed), nearby_trajs);

            if let Some(mut cluster) = cluster {
                // Expand the cluster to include all density-reachable segments
                self.expand_segment_cluster(&mut cluster, nearby_trajs);
                cluster_group.push(cluster);
            }
            // If no cluster forms, continue to next segment (not dense enough)
        }

        return cluster_group;
    }
}

impl TraclusAlgorithm for SerialTraclusDL {
    // ============================================================
    // Shared Data Accessors
    // ============================================================
    fn args(&self) -> &TraclusArgs {
        &self.args
    }

    fn stop_flag(&self) -> &Option<StopFlag> {
        &self.stop_flag
    }

    fn set_stop_flag(&mut self, stop_flag: StopFlag) {
        self.stop_flag = Some(stop_flag);
    }

    // ============================================================
    // Required Method
    // ============================================================

    /// Performs a version of DBSCAN clustering on trajectory segments organized in angle-based buckets.
    /// Implements the main clustering logic for the Serial TraClusDL algorithm.
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) -> bool {
        // Phase 1: serial discovery
        self.emit_start_clustering(raw_trajectories);
        self.complete_serial_clustering(raw_trajectories, clustered_trajectories);

        if self.is_stopped() {
            return false;
        }
        self.emit_complete_clustering();

        // Phase 2: serial fill in non-clustered segments
        self.fill_non_clustered_segments(raw_trajectories, clustered_trajectories);

        // Phase 3: create corridors from clusters and finalize non-clustered segments
        self.emit_start_remove_duplicates(clustered_trajectories);
        self.create_corridors(clustered_trajectories);

        if self.is_stopped() {
            return false;
        }
        self.emit_complete_remove_duplicates();
        return true;
    }
}
