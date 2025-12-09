use crate::cluster::{cluster::Cluster, cluster_member::ClusterMember};

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl std::ops::Mul<f64> for Point {
    type Output = Point;

    fn mul(self, scalar: f64) -> Point {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Debug)]
pub struct Segment {
    pub id: usize,
    pub start: Point,
    pub middle: Point,
}

impl Segment {
    pub fn new(id: usize, start: Point, end: Point) -> Self {
        let middle: Point = Point {
            x: (start.x + end.x) / 2.0,
            y: (start.y + end.y) / 2.0,
        };
        Self { id, start, middle }
    }

    pub fn get_end(&self) -> Point {
        let end: Point = Point {
            x: self.start.x + 2.0 * (self.middle.x - self.start.x),
            y: self.start.y + 2.0 * (self.middle.y - self.start.y),
        };
        end
    }

    pub fn get_length(&self) -> f64 {
        let end: Point = self.get_end();
        let dx: f64 = end.x - self.start.x;
        let dy: f64 = end.y - self.start.y;
        (dx * dx + dy * dy).sqrt()
    }
}

pub struct Corridor {
    pub id: usize,
    pub weight: u32,
    pub start: Point,
    pub end: Point,
}

impl Corridor {
    pub fn new(cluster: &Cluster, id: usize) -> Self {
        let (start, end) = Self::weighted_average(cluster);
        let weight: u32 = cluster.total_weight;
        Self {
            id,
            weight,
            start,
            end,
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
