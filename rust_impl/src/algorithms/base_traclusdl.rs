use crate::{
    clustering::{
        cluster::Cluster,
        cluster_member::{ClusterMember, ClusterSeed},
    },
    geometry::{segment::Segment, trajectory::Trajectory},
    io::traclus_args::TraclusArgs,
    storage::{clustered_trajectories::ClusteredTrajectories, raw_trajectories::RawTrajectories},
};

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

    // ============================================================
    // Required Methods (Must Be Implemented by Implementations)
    // ============================================================

    /// Phase 1.1: Performs DB-SCAN clustering on trajectory segments.
    ///
    /// This is the main method to partitions the trajectory into clusters
    /// based constraints.
    /// # Arguments
    /// * `raw_trajectories` - The raw trajectory storage containing all trajectories
    /// * `clustered_trajectories` - The clustered trajectory storage to populate with clusters
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    );

    /// Phase 1.2: Clusters an individual trajectory against nearby trajectories.
    ///
    /// # Arguments
    /// * `traj_seed` - The trajectory to use as a clustering seed
    /// * `nearby_trajs` - Vector of nearby trajectories to consider for clustering
    /// # Returns
    /// * A vector of clusters formed from the trajectory segments
    fn individual_trajectory_clustering(
        &self,
        traj_seed: &Trajectory,
        nearby_trajs: &Vec<&Trajectory>,
    ) -> Vec<Cluster>;

    /// Phase 2: Creates representative corridors from trajectory clusters.
    ///
    /// This phase generates corridors from the clustered trajectory segments.
    /// # Arguments
    /// * `clustered_trajectories` - The clustered trajectory storage containing clusters to process
    fn create_corridors(&self, clustered_trajectories: &mut ClusteredTrajectories);

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

            // Add qualifying segment as a candidate
            let segment = nearby_traj.segment(segment_id).unwrap();
            let candidate = ClusterMember::new(
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
}
