use crate::clustering::cluster::Cluster;
use crate::clustering::cluster_member::ClusterMember;
use crate::clustering::corridor::Corridor;
use crate::geometry::trajectory::Trajectory;
use crate::io::traclus_args::TraclusArgs;
use crate::storage::priority_queue::PriorityQueueCluster;

pub struct ClusteredTrajectories {
    clusters: PriorityQueueCluster,
    pub corridors: Vec<Corridor>,
}

impl ClusteredTrajectories {
    pub fn new() -> Self {
        Self {
            clusters: PriorityQueueCluster::new(),
            corridors: Vec::new(),
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

    pub fn fill_non_clustered_segments(&mut self, trajectory: &Trajectory) {
        for segment in trajectory.segments_iter() {
            let cluster_member: ClusterMember = ClusterMember::new_from_traj(trajectory, segment);
            self.clusters
                .non_clustered_segments
                .push(Box::new(cluster_member));
        }
    }

    pub fn pop_completed_cluster(&mut self, args: &TraclusArgs) -> Option<Box<Cluster>> {
        self.clusters.pop_and_clean(args.min_density)
    }

    pub fn list_non_clustered_segments(&self) -> &Vec<Box<ClusterMember>> {
        &self.clusters.non_clustered_segments
    }
}
