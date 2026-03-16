use crate::gui::app_events::ComputationEvent;
use crate::io::args::TraclusArgs;
use crate::utils::gui_parallel_runner::StopFlag;

use super::super::geometry::trajectory::Trajectory;
use super::super::objects::cluster::Cluster;
use super::super::objects::corridor::Corridor;
use super::super::storage::{
    clustered_trajectories::ClusteredTrajectories,
    raw_trajectories::{Bucket, RawTrajectories},
};
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
    fn create_corridors(
        &self,
        clustered_trajectories: &mut ClusteredTrajectories,
        emitter: &mut ComputationEvent,
    ) {
        let mut num_last_elements: usize = clustered_trajectories.get_size_priority_queue();

        while let Some(completed_cluster) =
            clustered_trajectories.pop_and_clean(self.args.min_density)
        {
            let index_corridor: usize = clustered_trajectories.corridors.len();
            let corridor: Corridor = Corridor::new(*completed_cluster, index_corridor);
            clustered_trajectories.corridors.push(corridor);

            let num_current_elements: usize = clustered_trajectories.get_size_priority_queue();
            self.tick_remove_duplicates(emitter, num_last_elements, num_current_elements);
            num_last_elements = num_current_elements;
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
        emitter: &mut ComputationEvent,
    ) {
        // Phase 1: parallel discovery
        self.emit_start_clustering(raw_trajectories, emitter);
        let results: Vec<Vec<Cluster>> = self.complete_parallel_clustering(raw_trajectories);
        self.emit_complete_clustering(emitter);

        // Phase 2: serial fill in non-clustered segments
        self.fill_non_clustered_segments(raw_trajectories, clustered_trajectories);

        // Phase 3: serial commit (regroup clusters)
        for clusters in results {
            clustered_trajectories.add_list_cluster(clusters);
        }

        // Phase 4: create corridors from clusters and finalize non-clustered segments
        self.emit_start_remove_duplicates(clustered_trajectories, emitter);
        self.create_corridors(clustered_trajectories, emitter);
        self.emit_complete_remove_duplicates(emitter);
    }
}
