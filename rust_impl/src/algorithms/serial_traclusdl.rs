use crate::{
    algorithms::base_traclusdl::TraclusAlgorithm,
    clustering::{cluster::Cluster, corridor::Corridor},
    geometry::trajectory::Trajectory,
    io::traclus_args::TraclusArgs,
    storage::{clustered_trajectories::ClusteredTrajectories, raw_trajectories::RawTrajectories},
};

pub struct SerialTraclusDL {
    args: TraclusArgs,
}

impl SerialTraclusDL {
    pub fn new(args: TraclusArgs) -> Self {
        Self { args }
    }
}

impl TraclusAlgorithm for SerialTraclusDL {
    // ============================================================
    // Shared Data Accessors
    // ============================================================
    fn args(&self) -> &TraclusArgs {
        &self.args
    }

    // ============================================================
    // Required Methods (Must Be Implemented)
    // ============================================================

    // TODO: optimize with maybe the reference of the nearby_trajectories vector instead of the iterator.collect()
    /// Performs a version of DBSCAN clustering on trajectory segments organized in angle-based buckets.
    ///
    /// This function:
    /// 1. Iterates through all trajectory segments in angle-ordered buckets
    /// 2. Attempts to form initial clusters from seed segments
    /// 3. Expands valid clusters by finding nearby dense regions
    /// 4. Set a collection of all discovered clusters inside the clustered trajectory storage
    /// 5. Marks non-clustered segments for later processing
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) {
        for bucket in &raw_trajectories.traj_buckets {
            // Get nearby trajectories for this angle bucket: contains all trajectories within angle range
            let nearby_trajs: Vec<&Trajectory> = raw_trajectories
                .iter_nearby_angle(bucket.angle_start)
                .collect();

            for traj_seed in &bucket.trajectories {
                // Cluster segments from this trajectory using nearby trajectories
                let clusters: Vec<Cluster> =
                    self.individual_trajectory_clustering(traj_seed, &nearby_trajs);
                clustered_trajectories.add_list_cluster(clusters);

                // Fill all segments to be treated as non-clustered later
                clustered_trajectories.fill_non_clustered_segments(traj_seed);
            }
        }
    }

    /// Clusters all segments in a trajectory attempting to form and expand clusters.
    ///
    /// For each segment:
    /// - Attempts to create an initial cluster if density requirements are met
    /// - Expands the cluster to include all reachable segments
    /// - Stores the completed cluster
    #[inline]
    fn individual_trajectory_clustering(
        &self,
        traj_seed: &Trajectory,
        nearby_trajs: &Vec<&Trajectory>,
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

    fn create_corridors(&self, clustered_trajectories: &mut ClusteredTrajectories) {
        while let Some(completed_cluster) = clustered_trajectories.pop_completed_cluster(&self.args)
        {
            let index_corridor: usize = clustered_trajectories.corridors.len();
            clustered_trajectories
                .corridors
                .push(Corridor::new(&completed_cluster, index_corridor));
        }

        for non_clust_seg in clustered_trajectories.list_non_clustered_segments() {
            print!(
                "Non-clustered segment: {:?}, {:?}\n",
                non_clust_seg.traj_id, non_clust_seg.segment_id
            );
        }
    }
}
