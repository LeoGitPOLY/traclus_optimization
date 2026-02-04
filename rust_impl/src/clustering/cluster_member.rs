use crate::geometry::{point::Point, trajectory::Trajectory, segment::Segment};

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

    pub fn end_point(&self) -> Point {
        Point {
            x: self.start.x + 2.0 * (self.center.x - self.start.x),
            y: self.start.y + 2.0 * (self.center.y - self.start.y),
        }
    }

    pub fn angle(&self) -> f64 {
        let dx = self.center.x - self.start.x;
        let dy = self.center.y - self.start.y;
        dy.atan2(dx).to_degrees()
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
