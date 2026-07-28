use crate::{geometry::point::Point, utils::data_type::decimal_i32::DecimalI32};

#[derive(Debug, Clone)]
pub struct PointI32Proj {
    pub x: DecimalI32,
    pub y: DecimalI32,
}

impl PointI32Proj {
    pub fn from_point(point: &Point) -> Self {
        Self {
            x: DecimalI32::from_f64(point.x),
            y: DecimalI32::from_f64(point.y),
        }
    }

    pub fn to_point(&self) -> Point {
        let x: f64 = self.x.to_f64();
        let y: f64 = self.y.to_f64();
        Point { x, y }
    }

    pub fn power_distance_u64(&self, point_i32: &PointI32Proj) -> u64 {
        let dist: PointI32Proj = self.subtract(point_i32);
        dist.x.power_u64() + dist.y.power_u64()
    }

    pub fn subtract(&self, other: &PointI32Proj) -> PointI32Proj {
        PointI32Proj {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
