use std::slice;

use crate::geometry::segment::Segment;
use crate::io::args::TraclusArgs;
use crate::utils::events::event_singleton::emit_timed_perf;
use crate::utils::gui_parallel_runner::StopFlag;

use super::base_traclusdl::TICK_EVERY;
use super::base_traclusdl::TraclusAlgorithm;
use crate::geometry::trajectory::Trajectory;
use crate::objects::cluster::Cluster;
use crate::objects::corridor::Corridor;
use crate::storage::{
    clustered_trajectories::ClusteredTrajectories,
    raw_trajectories::{Bucket, RawTrajectories},
};

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

    /// Completes the parallel clustering using Rayon by iterating over angle buckets
    /// Each trajectory inside each bucket is computed in parallel
    ///
    /// # Arguments
    /// * `raw_trajectories` - The raw trajectory storage containing all trajectories
    /// * `clustered_trajectories` - The clustered trajectory storage to populate with clusters
    fn complete_parallel_clustering_standalone(
        &self,
        raw_trajectories: &RawTrajectories,
    ) -> Vec<Vec<Cluster>> {
        // Serial iterator over angle buckets
        let bucket_serial_iter: slice::Iter<'_, Bucket> = raw_trajectories.traj_buckets.iter();
        let mut results: Vec<Vec<Cluster>> = Vec::new();

        for bucket in bucket_serial_iter {
            // Get a copy of nearby trajectories for this angle bucket
            // Since this is not mutable, this is read-only and thread-safe
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
        }
        results
    }

    // fn complete_parallel_clustering_interact(
    //     &self,
    //     raw_trajectories: &RawTrajectories,
    // ) -> Vec<Vec<Cluster>> {
    //     // Flatten all buckets into one iterator of (bucket_angle, trajectory) pairs
    //     // Drains bucket order: first bucket exhausted, then second, etc.
    //     let all_trajectories: Vec<(f64, &Trajectory)> = raw_trajectories
    //         .traj_buckets
    //         .iter()
    //         .flat_map(|bucket| {
    //             bucket
    //                 .trajectories
    //                 .iter()
    //                 .map(move |traj| (bucket.angle_start, traj))
    //         })
    //         .collect();

    //     let mut results: Vec<Vec<Cluster>> = Vec::new();

    //     // Process one chunk of TICK_EVERY trajectories at a time
    //     for chunk in all_trajectories.chunks(TICK_EVERY) {
    //         // Stop early if requested
    //         if self.is_stopped() {
    //             break;
    //         }

    //         // Process this chunk in parallel — each trajectory gets its nearby set
    //         let chunk_results: Vec<Vec<Cluster>> = chunk
    //             .par_iter()
    //             .map(|(angle_start, traj)| {
    //                 let thread_index: Option<usize> = rayon::current_thread_index();
    //                 emit_timed_perf("Clustering", true, thread_index);

    //                 let nearby_trajs: Vec<&Trajectory> =
    //                     raw_trajectories.iter_nearby_angle(*angle_start).collect();
    //                 let clusters: Vec<Cluster> =
    //                     self.individual_trajectory_clustering(traj, &nearby_trajs);
    //                 emit_timed_perf("Clustering", false, thread_index);
    //                 clusters
    //             })
    //             .collect();

    //         results.extend(chunk_results);

    //         // Tick after the chunk completes (count = actual chunk size, handles last chunk)
    //         self.tick_clustering(&mut chunk.len());
    //     }

    //     results
    // }

    /// TODO COMMENTS
    fn individual_trajectory_clustering(
        &self,
        traj_seed: &Trajectory,
        nearby_trajs: &[Trajectory],
    ) -> Vec<Cluster> {
        // Parallelize over segments from this Trajectory using Rayon
        let traj_parallel_iter = traj_seed.segments_par_iter();

        let traj_results: Vec<Cluster> = traj_parallel_iter
            .filter_map(|seed_segment: &Segment| {
                // Cluster the all segment as seed with nearby trajectories
                let cluster: Option<Cluster> =
                    self.initial_segment_cluster((&seed_segment, traj_seed), nearby_trajs);

                // If cluster meats the requirements expand it
                if let Some(mut cluster) = cluster {
                    self.expand_segment_cluster(&mut cluster, nearby_trajs);
                    Some(cluster)
                } else {
                    None
                }
            })
            .collect();

        traj_results
    }

    /// Same logic as the serial version — unchanged
    /// Creates corridors for all clustered trajectories based on the clustering results
    /// # Arguments
    /// * `clustered_trajectories` - The clustered trajectory storage containing all clusters
    fn create_corridors(&self, clustered_trajectories: &mut ClusteredTrajectories) {
        let mut num_last_elements: usize = clustered_trajectories.get_size_priority_queue();

        while let Some(completed_cluster) = clustered_trajectories.pop_and_clean(&self.args) {
            let index_corridor: usize = clustered_trajectories.corridors.len();
            let corridor: Corridor = Corridor::new(completed_cluster, index_corridor);
            clustered_trajectories.corridors.push(corridor);

            let num_current_elements: usize = clustered_trajectories.get_size_priority_queue();
            self.tick_remove_duplicates(num_last_elements, num_current_elements);
            num_last_elements = num_current_elements;

            // Check for stop signal to bail out early
            if self.is_stopped() {
                return;
            }
        }
        clustered_trajectories.take_non_clustered_segments();
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

    /// Performs a version of DBSCAN clustering on trajectory segments organized in angle-based buckets.
    /// Implements the main clustering logic for the parallel TraClusDL algorithm using Rayon for parallelism.
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) -> bool {
        // Phase 1: parallel discovery
        self.emit_start_clustering(raw_trajectories);
        let results: Vec<Vec<Cluster>> =
            self.complete_parallel_clustering_standalone(raw_trajectories);

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
