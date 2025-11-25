use crate::spatial::geometry::Point;
use crate::spatial::geometry::Segment;
use crate::spatial::input_od_line::InputODLine;

#[derive(Debug)]
pub struct Trajectory {
    pub id: usize,
    pub start: Point,
    pub end: Point,
    pub weight: u32,
    pub angle: f64,
    segments: Vec<Segment>,
}

impl Trajectory {
    pub fn new(input: InputODLine) -> Self {
        let angle: f64 = Self::get_spatial_angle(&input.start, &input.end);

        Self {
            id: input.line_id,
            start: input.start,
            end: input.end,
            weight: input.weight,
            angle,
            segments: Vec::new(),
        }
    }

    fn get_spatial_angle(start: &Point, end: &Point) -> f64 {
        let delta_y = end.y - start.y;
        let delta_x = end.x - start.x;
        let mut angle = delta_y.atan2(delta_x).to_degrees();
        angle = (angle * 100.0).round() / 100.0;

        if angle < 0.0 {
            angle += 360.0;
        }

        angle
    }

    fn get_spatial_length(start: &Point, end: &Point) -> f64 {
        let delta_y: f64 = end.y - start.y;
        let delta_x: f64 = end.x - start.x;
        (delta_x.powi(2) + delta_y.powi(2)).sqrt()
    }

    pub fn create_segments(&mut self) {
        // Placeholder example:
        let segment = Segment::new(0, self.start.clone(), self.end.clone());
        self.segments.push(segment);
    }

    pub fn to_str(&self) -> String {
        format!(
            "Trajectory ID: {}, Start: ({}, {}), End: ({}, {}), Weight: {}, Angle: {}",
            self.id, self.start.x, self.start.y, self.end.x, self.end.y, self.weight, self.angle
        )
    }
}
