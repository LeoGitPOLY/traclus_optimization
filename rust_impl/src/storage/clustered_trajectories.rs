use crate::clustering::cluster::Cluster;
use crate::clustering::cluster_member::ClusterMember;
use crate::clustering::corridor::Corridor;
use crate::geometry::trajectory::Trajectory;
use crate::io::args::TraclusArgs;
use crate::storage::priority_queue::PriorityQueueCluster;

pub struct ClusteredTrajectories {
    clusters: PriorityQueueCluster,
    pub corridors: Vec<Corridor>,
    pub non_clustered_segments: Vec<ClusterMember>,
}

impl ClusteredTrajectories {
    pub fn new() -> Self {
        Self {
            clusters: PriorityQueueCluster::new(),
            corridors: Vec::new(),
            non_clustered_segments: Vec::new(),
        }
    }

    pub fn add_cluster(&mut self, cluster: Cluster) {
        self.clusters.push(cluster);
    }

    pub fn add_list_cluster(&mut self, clusters: Vec<Cluster>) {
        for cluster in clusters {
            self.add_cluster(cluster);
        }
    }

    pub fn finalize_corridors(&mut self, args: &TraclusArgs) {
        while let Some(completed_cluster) = self.clusters.pop_and_clean(args.min_density) {
            let index_corridor: usize = self.corridors.len();
            let corridor: Corridor = Corridor::new(*completed_cluster, index_corridor);
            self.corridors.push(corridor);
        }
        self.non_clustered_segments = std::mem::take(&mut self.clusters.non_clustered_segments);
        self.clusters = PriorityQueueCluster::new();
    }

    pub fn fill_non_clustered_segments(&mut self, trajectory: &Trajectory) {
        for segment in trajectory.segments_iter() {
            let cluster_member: ClusterMember = ClusterMember::new_from_traj(trajectory, segment);
            self.clusters.non_clustered_segments.push(cluster_member);
        }
    }

    // Provides an iterator over all cluster members in all corridors, along with their corridor index
    // Corridor index is -1 for non-clustered segments
    pub fn get_all_cluster_members_iter(&self) -> impl Iterator<Item = (i32, &ClusterMember)> {
        // Iterate over all corridors and their members, yielding (corridor_id, cluster_member)
        let clustered = self
            .corridors
            .iter()
            .enumerate()
            .flat_map(|(corridor_idx, corridor)| {
                corridor
                    .cluster
                    .get_all_members_iter()
                    .map(move |cm| (corridor_idx as i32, cm))
            });
        // Iterate over non-clustered segments, yielding (-1, cluster_member)
        let non_clustered = self.non_clustered_segments.iter().map(|cm| (-1, cm));

        // Merge the two iterators
        clustered.chain(non_clustered)
    }
}
