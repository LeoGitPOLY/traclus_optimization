use crate::spatial::geometry::Point;

pub struct ClusterMember {
    pub trajectory_id: usize,
    pub segment_id: usize,
    pub center_point: Point,
}

impl ClusterMember {
    pub fn new(trajectory_id: usize, segment_id: usize, center_point: &Point) -> Self {
        Self {
            trajectory_id,
            segment_id,
            center_point: center_point.clone(),
        }
    }
}

pub struct Cluster {
    seed: ClusterMember,
    total_weight: u32,
    candidates: Vec<ClusterMember>,
    members: Vec<ClusterMember>,
}

impl Cluster {
    pub fn new(seed: ClusterMember, total_weight: u32, candidates: Vec<ClusterMember>) -> Self {
        Self {
            seed,
            total_weight,
            candidates,
            members: Vec::new(),
        }
    }
    pub fn print_info(&self) {
        println!(
            "Cluster Seed Traj ID: {}, Segment ID: {}, Total Weight: {}, Num Candidates: {}",
            self.seed.trajectory_id,
            self.seed.segment_id,
            self.total_weight,
            self.candidates.len()
        );
    }
}

pub struct ClusteredTrajStore {
    // Placeholder for future implementation
}

impl ClusteredTrajStore {
    pub fn new() -> Self {
        Self {
            // Placeholder for future implementation
        }
    }

    pub fn order_clusters(&mut self) {
        // Placeholder for future implementation
    }
}
