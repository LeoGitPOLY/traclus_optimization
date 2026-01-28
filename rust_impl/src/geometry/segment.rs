use crate::geometry::point::Point;

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
