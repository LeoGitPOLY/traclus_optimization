#[derive(Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub id: usize,
    pub start: Point,
    pub middle: Point,
    pub end: Point,
}

impl Segment {
    pub fn new(id: usize, start: Point, end: Point) -> Self {
        let middle: Point = Point {
            x: (start.x + end.x) / 2.0,
            y: (start.y + end.y) / 2.0,
        };
        Self {
            id,
            start,
            middle,
            end,
        }
    }
}
