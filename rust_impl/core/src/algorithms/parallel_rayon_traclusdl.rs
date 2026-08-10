// parallel_rayon_traclusdl.rs — Rayon-parallel TraClus over trajectories and segments
use std::slice;

use super::base_traclusdl::TraclusAlgorithm;
use crate::geometry::segment::Segment;
use crate::geometry::trajectory::Trajectory;
use crate::io::args::TraclusArgs;
use crate::objects::cluster::Cluster;
use crate::storage::{
    clustered_trajectories::ClusteredTrajectories,
    raw_trajectories::{Bucket, RawTrajectories},
};
use crate::utils::events::event_singleton::emit_timed_perf;
use crate::utils::gui_parallel_runner::StopFlag;

use rayon::prelude::*;
use rayon::slice::Iter;

pub struct ParallelRayonTraclusDL {
    args: TraclusArgs,
    stop_flag: Option<StopFlag>,
}

impl ParallelRayonTraclusDL {
    pub fn new(args: TraclusArgs) -> Self {
        Self {
            args,
            stop_flag: None,
        }
    }

    // Serial over buckets; parallel over trajectories within each bucket
    fn complete_parallel_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
    ) -> Vec<Vec<Cluster>> {
        // Serial iterator over angle buckets
        let bucket_serial_iter: slice::Iter<'_, Bucket> = raw_trajectories.traj_buckets.iter();
        let mut results: Vec<Vec<Cluster>> = Vec::new();

        for bucket in bucket_serial_iter {
            // Shared read-only nearby copy per bucket
            emit_timed_perf("Copy_Nearby_Trajectories", true, None);
            let nearby_trajs: Vec<Trajectory> =
                raw_trajectories.vec_nearby_angle(bucket.angle_start);
            emit_timed_perf("Copy_Nearby_Trajectories", false, None);

            // Parallelize over trajectories in this bucket using Rayon
            let traj_parallel_iter: Iter<'_, Trajectory> = bucket.trajectories.par_iter();
            let bucket_results: Vec<Vec<Cluster>> = traj_parallel_iter
                .map(|traj_seed: &Trajectory| {
                    let thread_index: Option<usize> = rayon::current_thread_index();

                    emit_timed_perf("Clustering", true, thread_index);
                    let clusters: Vec<Cluster> =
                        self.individual_trajectory_clustering(traj_seed, &nearby_trajs);
                    emit_timed_perf("Clustering", false, thread_index);

                    clusters
                })
                .collect::<Vec<_>>();

            // Commit the results for this bucket
            emit_timed_perf("Commiting", true, None);
            results.extend(bucket_results);
            emit_timed_perf("Commiting", false, None);

            // Stop early if requested
            if self.is_stopped() {
                break;
            }

            // Tick after every bucket (count = actual number of trajectories in this bucket)
            self.tick_clustering(bucket.trajectories.len());
        }
        results
    }

    /// Clusters an individual trajectory against nearby trajectories.
    /// For each segment (treated in parallel):
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
        // Parallelize over segments from this Trajectory using Rayon
        let traj_parallel_iter = traj_seed.segments_par_iter();

        let traj_results: Vec<Cluster> = traj_parallel_iter
            .filter_map(|seed_segment: &Segment| {
                // Try to form an initial cluster from this seed segment
                let cluster: Option<Cluster> =
                    self.initial_segment_cluster((&seed_segment, traj_seed), nearby_trajs);

                if let Some(mut cluster) = cluster {
                    // Expand the cluster to include all density-reachable segments
                    self.expand_segment_cluster(&mut cluster, nearby_trajs);
                    Some(cluster)
                } else {
                    // If no cluster forms, continue to next segment (not dense enough)
                    None
                }
            })
            .collect();

        traj_results
    }
}

impl TraclusAlgorithm for ParallelRayonTraclusDL {
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
    /// Implements the main clustering logic for the parallel TraClusDL algorithm using Rayon for parallelism.
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) -> bool {
        // Phase 1: parallel discovery
        self.emit_start_clustering(raw_trajectories);
        let results: Vec<Vec<Cluster>> = self.complete_parallel_clustering(raw_trajectories);

        if self.is_stopped() {
            return false;
        }
        self.emit_complete_clustering();

        // Phase 2: serial fill in non-clustered segments
        self.fill_non_clustered_segments(raw_trajectories, clustered_trajectories);

        // Phase 3: serial commit (regroup clusters)
        emit_timed_perf("Commiting_Results", true, None);
        for clusters in results {
            clustered_trajectories.add_list_cluster(clusters);
        }
        emit_timed_perf("Commiting_Results", false, None);

        // Phase 4: create corridors from clusters and finalize non-clustered segments
        self.emit_start_remove_duplicates(clustered_trajectories);
        self.create_corridors(clustered_trajectories);

        if self.is_stopped() {
            return false;
        }
        self.emit_complete_remove_duplicates();
        return true;
    }
}
