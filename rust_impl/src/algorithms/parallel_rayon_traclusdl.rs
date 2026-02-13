use crate::{
    algorithms::base_traclusdl::TraclusAlgorithm,
    clustering::cluster::Cluster,
    geometry::trajectory::Trajectory,
    io::traclus_args::TraclusArgs,
    storage::{
        clustered_trajectories::ClusteredTrajectories,
        raw_trajectories::{Bucket, RawTrajectories},
    },
};

use rayon::prelude::*;
use rayon::slice::Iter;

pub struct ParallelRayonTraclusDL {
    args: TraclusArgs,
}

impl ParallelRayonTraclusDL {
    pub fn new(args: TraclusArgs) -> Self {
        Self { args }
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

    /// Same logic as the serial version — unchanged
    fn create_corridors(&self, clustered_trajectories: &mut ClusteredTrajectories) {
        clustered_trajectories.finalize_corridors(self.args());
    }
}

impl TraclusAlgorithm for ParallelRayonTraclusDL {
    fn args(&self) -> &TraclusArgs {
        &self.args
    }

    /// Performs a version of DBSCAN clustering on trajectory segments organized in angle-based buckets.
    /// Implements the main clustering logic for the parallel TraClusDL algorithm using Rayon for parallelism.
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) {
        // Phase 1: parallel discovery
        let results: Vec<Vec<Cluster>> = self.complete_parallel_clustering(raw_trajectories);

        // Phase 2: serial fill in non-clustered segments
        self.fill_non_clustered_segments(raw_trajectories, clustered_trajectories);

        // Phase 3: serial commit (regroup clusters)
        for clusters in results {
            clustered_trajectories.add_list_cluster(clusters);
        }

        // Phase 3: create corridors from clusters and finalize non-clustered segments
        self.create_corridors(clustered_trajectories);
    }
}
