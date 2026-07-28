use std::f64::consts::PI;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::utils::data_type::angle_u16::AngleU16;

use super::input_od_line::InputODLine;
use super::point::Point;
use super::segment::Segment;

#[derive(Debug, Clone)]
pub struct Trajectory {
    pub id: usize,
    pub start: Point,
    pub end: Point,
    pub weight: u32,
    pub angle: AngleU16,
    segments: Vec<Segment>,
}

impl Trajectory {
    pub fn new(input: InputODLine, seg_size: f64) -> Self {
        let angle: AngleU16 = Self::get_spatial_angle(&input.start, &input.end);

        let mut traj: Trajectory = Self {
            id: input.line_id,
            start: input.start,
            end: input.end,
            weight: input.weight,
            angle,
            segments: Vec::new(),
        };

        traj.make_segments(seg_size);
        traj
    }

    fn get_spatial_angle(start: &Point, end: &Point) -> AngleU16 {
        let delta_y: f64 = end.y - start.y;
        let delta_x: f64 = end.x - start.x;
        let angle_deg: f64 = delta_y.atan2(delta_x).to_degrees();
        AngleU16::from_degrees(angle_deg)
    }

    fn get_spatial_length(&self) -> f64 {
        let dx: f64 = self.end.x - self.start.x;
        let dy: f64 = self.end.y - self.start.y;
        (dx * dx + dy * dy).sqrt()
    }

    // Returns the minimum distance from a point to the trajectory
    // and the index of the segment that is closest to the point.
    pub fn distance_to_point(&self, point: &Point) -> (f64, usize) {
        let px: f64 = point.x;
        let py: f64 = point.y;
        let x1: f64 = self.start.x;
        let y1: f64 = self.start.y;
        let dx: f64 = self.end.x - self.start.x;
        let dy: f64 = self.end.y - self.start.y;

        if dx == 0.0 && dy == 0.0 {
            let min_distance: f64 = ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
            return (min_distance, 0);
        }

        let t: f64 = ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy);
        let t: f64 = t.clamp(0.0, 1.0);

        let near: Point = Point {
            x: x1 + t * dx,
            y: y1 + t * dy,
        };

        let min_distance: f64 = ((px - near.x).powi(2) + (py - near.y).powi(2)).sqrt();
        let traj_length: f64 = self.get_spatial_length();
        let seg_length: f64 = self.segments.first().unwrap().get_length();

        let index_seg: usize = (t * (traj_length / seg_length)) as usize;

        (min_distance, index_seg)
    }

    //pub fn power_distance_to_point(&self, point: &PointI32Proj) -> (u64, usize) {}

    pub fn make_segments(&mut self, segment_length: f64) {
        self.segments.clear();

        let angle_rad: f64 = self.angle.to_degrees() * PI / 180.0;
        let xstep: f64 = segment_length * angle_rad.cos();
        let ystep: f64 = segment_length * angle_rad.sin();

        let length: f64 = self.get_spatial_length();

        let nsegs: usize = ((length / segment_length) - 1e-9).ceil() as usize;
        let base_x: f64 = self.start.x;
        let base_y: f64 = self.start.y;

        for i in 0..nsegs {
            let start_x: f64 = base_x + (i as f64) * xstep;
            let start_y: f64 = base_y + (i as f64) * ystep;

            let end_x: f64 = base_x + ((i + 1) as f64) * xstep;
            let end_y: f64 = base_y + ((i + 1) as f64) * ystep;

            let seg_start: Point = Point {
                x: start_x,
                y: start_y,
            };
            let seg_end: Point = Point { x: end_x, y: end_y };

            let segment: Segment = Segment::new(i, seg_start, seg_end);
            self.segments.push(segment);
        }
    }

    pub fn segments_iter(&self) -> impl Iterator<Item = &Segment> {
        self.segments.iter()
    }

    pub fn segments_par_iter(&self) -> impl ParallelIterator<Item = &Segment> {
        self.segments.par_iter()
    }

    pub fn segment(&self, index: usize) -> Option<&Segment> {
        self.segments.get(index)
    }

    #[allow(unused)]
    pub fn print_info(&self) -> String {
        format!(
            "Trajectory ID: {}, Start: ({}, {}), End: ({}, {}), Weight: {}, Angle: {}",
            self.id, self.start.x, self.start.y, self.end.x, self.end.y, self.weight, self.angle
        )
    }
}

// UNIT TESTS : TO BE DELETED
#[cfg(test)]
mod tests {
    use crate::utils::data_type::point_i32_proj::PointI32Proj;

    use super::*;

    #[test]
    fn distance_to_point_horizontal_line() {
        let input: InputODLine = InputODLine {
            line_id: 0,
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 10.0, y: 0.0 },
            weight: 1,
        };

        // Segment size of 5 gives two segments.
        let traj: Trajectory = Trajectory::new(input, 5.0);
        let point: Point = Point { x: 3.0, y: 4.0 };

        let (distance, segment_index) = traj.distance_to_point(&point);

        assert!((distance - 4.0).abs() < 1e-9);
        assert_eq!(segment_index, 0);
    }

    #[test]
    fn run_tests_on_pointi32proj() {
        let point_1: Point = Point {
            x: -0.0499,
            y: -0.0499,
        };
        let mut point_2: Point = Point {
            x: 100.0499,
            y: 100.0499,
        };
        let point_max: Point = Point {
            x: 50_000.0499,
            y: 50_000.0499,
        };
        point_2 = point_max.clone();

        let point_proj_1: PointI32Proj = PointI32Proj::from_point(&point_1);
        let point_proj_2: PointI32Proj = PointI32Proj::from_point(&point_2);

        println!("Projected Point 1: {:?}", point_proj_1);
        println!("Projected Point 2: {:?}", point_proj_2);

        // Test subtraction of projected points
        let real_sub: Point = Point {
            x: point_1.x - point_2.x,
            y: point_1.y - point_2.y,
        };
        let point_sub: PointI32Proj = point_proj_1.subtract(&point_proj_2);
        println!("Real Point Subtraction: {:?}", real_sub);
        println!("Projected Point Subtraction: {:?}", point_sub);

        let real_power_dist: f64 =
            (point_1.x - point_2.x).powi(2) + (point_1.y - point_2.y).powi(2);
        let proj_power_dist: u64 = point_proj_1.power_distance_u64(&point_proj_2);
        println!("Real Power Distance: {}", real_power_dist);
        println!("Projected Power Distance: {}", proj_power_dist);

        println!("Real Distance: {}", real_power_dist.sqrt());
        println!("Projected Distance: {}", (proj_power_dist as f64).sqrt());
    }
}
