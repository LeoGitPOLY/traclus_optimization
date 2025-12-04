use crate::spatial::geometry::Point;

pub struct ClusterMember {
    pub traj_id: usize,
    pub segment_id: usize,
    pub weight: u32,
    pub center_point: Point,
}

impl ClusterMember {
    pub fn new(traj_id: usize, segment_id: usize, weight: u32, center_point: Point) -> Self {
        Self {
            traj_id,
            segment_id,
            weight,
            center_point,
        }
    }
}

pub struct ClusterSeed {
    pub cm: ClusterMember,
    pub angle: f64,
}

impl ClusterSeed {
    pub fn new(cm: ClusterMember, angle: f64) -> Self {
        Self { cm, angle }
    }
}
