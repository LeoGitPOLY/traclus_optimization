use crate::spatial::{
    geometry::{Point, Segment},
    trajectory::Trajectory,
};

pub struct ClusterMember {
    pub traj_id: usize,
    pub segment_id: usize,
    pub weight: u32,
    pub center: Point,
    pub start: Point,
}

impl ClusterMember {
    pub fn new(
        traj_id: usize,
        segment_id: usize,
        weight: u32,
        center_point: Point,
        start_point: Point,
    ) -> Self {
        Self {
            traj_id,
            segment_id,
            weight,
            center: center_point,
            start: start_point,
        }
    }
    pub fn new_from_candidate(cm: &ClusterMember) -> Self {
        Self {
            traj_id: cm.traj_id,
            segment_id: cm.segment_id,
            weight: cm.weight,
            center: cm.center.clone(),
            start: cm.start.clone(),
        }
    }
    pub fn new_from_traj(traj: &Trajectory, seg: &Segment) -> Self {
        Self {
            traj_id: traj.id,
            segment_id: seg.id,
            weight: traj.weight,
            center: seg.middle.clone(),
            start: seg.start.clone(),
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
