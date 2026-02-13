use crate::{
    clustering::{cluster::Cluster, cluster_member::ClusterMember},
    geometry::point::Point,
};

pub struct Corridor {
    pub id: usize,
    pub weight: u32,
    pub start: Point,
    pub end: Point,
    pub cluster: Cluster,
}

impl Corridor {
    pub fn new(cluster: Cluster, id: usize) -> Self {
        let (start, end) = Self::weighted_average(&cluster);
        let weight: u32 = cluster.total_weight;
        Self {
            id,
            weight,
            start,
            end,
            cluster,
        }
    }

    pub fn weighted_average(cluster: &Cluster) -> (Point, Point) {
        let mut weighted_start: Point = cluster.seed.cm.start * (cluster.seed.cm.weight as f64);
        let mut weighted_end: Point = Self::get_weighted_end(&cluster.seed.cm);

        for member in &cluster.members {
            weighted_start = weighted_start + (member.start * (member.weight as f64));
            weighted_end = weighted_end + Self::get_weighted_end(member);
        }

        let total_weight_divider: f64 = 1.0 / cluster.total_weight as f64;
        (
            weighted_start * total_weight_divider,
            weighted_end * total_weight_divider,
        )
    }

    #[inline]
    pub fn get_weighted_end(member: &ClusterMember) -> Point {
        Point {
            x: member.center.x + (member.center.x - member.start.x),
            y: member.center.y + (member.center.y - member.start.y),
        } * (member.weight as f64)
    }
}
