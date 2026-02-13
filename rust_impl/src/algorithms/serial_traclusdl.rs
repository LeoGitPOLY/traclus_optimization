use crate::{
    algorithms::base_traclusdl::TraclusAlgorithm,
    clustering::cluster::Cluster,
    geometry::trajectory::Trajectory,
    io::traclus_args::TraclusArgs,
    storage::{clustered_trajectories::ClusteredTrajectories, raw_trajectories::RawTrajectories},
};

pub struct SerialTraclusDL {
    args: TraclusArgs,
}

impl TraclusAlgorithm for SerialTraclusDL {
    // ============================================================
    // Shared Data Accessors
    // ============================================================
    fn args(&self) -> &TraclusArgs {
        &self.args
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
    ) {
        self.complete_serial_clustering(raw_trajectories, clustered_trajectories);
        self.create_corridors(clustered_trajectories);
    }
}

impl SerialTraclusDL {
    pub fn new(args: TraclusArgs) -> Self {
        Self { args }
    }

    /// Completes the serial clustering process by iterating over angle buckets
    /// Clusters each trajectory and fills non-clustered segments
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

    /// Creates corridors for all clustered trajectories based on the clustering results
    /// # Arguments
    /// * `clustered_trajectories` - The clustered trajectory storage containing all clusters
    fn create_corridors(&self, clustered_trajectories: &mut ClusteredTrajectories) {
        clustered_trajectories.finalize_corridors(self.args());
    }
}
