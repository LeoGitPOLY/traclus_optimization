use crate::io::args::TraclusArgs;
use crate::utils::gui_parallel_runner::StopFlag;

use super::super::geometry::trajectory::Trajectory;
use super::super::objects::cluster::Cluster;
use super::super::objects::corridor::Corridor;
use super::super::storage::{
    clustered_trajectories::ClusteredTrajectories,
    raw_trajectories::{Bucket, RawTrajectories},
};
use super::base_traclusdl::TICK_EVERY;
use super::base_traclusdl::TraclusAlgorithm;

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
    fn complete_parallel_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
    ) -> Vec<Vec<Cluster>> {
        // Parallelize over angle buckets using Rayon
        let bucket_parallel_iter: Iter<'_, Bucket> = raw_trajectories.traj_buckets.par_iter();

        bucket_parallel_iter
            .flat_map(|bucket| {
                // Get nearby trajectories for this angle bucket: contains all trajectories within angle range
                let nearby_trajs: Vec<&Trajectory> = raw_trajectories
                    .iter_nearby_angle(bucket.angle_start)
                    .collect();

                // Parallelize over trajectories in this bucket using Rayon
                let traj_parallel_iter: Iter<'_, Trajectory> = bucket.trajectories.par_iter();
                traj_parallel_iter
                    .map(|traj_seed| {
                        self.individual_trajectory_clustering(traj_seed, &nearby_trajs)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn complete_parallel_clustering_v2(
        &self,
        raw_trajectories: &RawTrajectories,
    ) -> Vec<Vec<Cluster>> {
        // Flatten all buckets into one iterator of (bucket_angle, trajectory) pairs
        // Drains bucket order: first bucket exhausted, then second, etc.
        let all_trajectories: Vec<(f64, &Trajectory)> = raw_trajectories
            .traj_buckets
            .iter()
            .flat_map(|bucket| {
                bucket
                    .trajectories
                    .iter()
                    .map(move |traj| (bucket.angle_start, traj))
            })
            .collect();

        let mut results: Vec<Vec<Cluster>> = Vec::new();

        // Process one chunk of TICK_EVERY trajectories at a time
        for chunk in all_trajectories.chunks(TICK_EVERY) {
            // Stop early if requested
            if self.is_stopped() {
                break;
            }

            // Process this chunk in parallel — each trajectory gets its nearby set
            let chunk_results: Vec<Vec<Cluster>> = chunk
                .par_iter()
                .map(|(angle_start, traj)| {
                    let nearby_trajs: Vec<&Trajectory> =
                        raw_trajectories.iter_nearby_angle(*angle_start).collect();
                    self.individual_trajectory_clustering(traj, &nearby_trajs)
                })
                .collect();

            results.extend(chunk_results);

            // Tick after the chunk completes (count = actual chunk size, handles last chunk)
            self.tick_clustering(&mut chunk.len());
        }

        results
    }

    /// Same logic as the serial version — unchanged
    #[inline]
    fn individual_trajectory_clustering(
        &self,
        traj_seed: &Trajectory,
        nearby_trajs: &[&Trajectory],
    ) -> Vec<Cluster> {
        let mut cluster_group = Vec::new();

        for seed_segment in traj_seed.segments_iter() {
            let cluster =
                self.initial_segment_cluster((&seed_segment, traj_seed), &nearby_trajs.to_vec());

            if let Some(mut cluster) = cluster {
                self.expand_segment_cluster(&mut cluster, &nearby_trajs.to_vec());
                cluster_group.push(cluster);
            }
        }

        cluster_group
    }

    /// Same logic as the serial version — unchanged
    /// Creates corridors for all clustered trajectories based on the clustering results
    /// # Arguments
    /// * `clustered_trajectories` - The clustered trajectory storage containing all clusters
    fn create_corridors(&self, clustered_trajectories: &mut ClusteredTrajectories) {
        let mut num_last_elements: usize = clustered_trajectories.get_size_priority_queue();

        while let Some(completed_cluster) =
            clustered_trajectories.pop_and_clean(self.args.min_density)
        {
            let index_corridor: usize = clustered_trajectories.corridors.len();
            let corridor: Corridor = Corridor::new(*completed_cluster, index_corridor);
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
        let results: Vec<Vec<Cluster>> = self.complete_parallel_clustering_v2(raw_trajectories);

        if self.is_stopped() {
            return false;
        }
        self.emit_complete_clustering();

        // Phase 2: serial fill in non-clustered segments
        self.fill_non_clustered_segments(raw_trajectories, clustered_trajectories);

        // Phase 3: serial commit (regroup clusters)
        for clusters in results {
            clustered_trajectories.add_list_cluster(clusters);
        }

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
