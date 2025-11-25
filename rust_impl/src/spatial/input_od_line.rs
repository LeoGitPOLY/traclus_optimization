use crate::spatial::geometry::Point;

#[derive(Debug)]
pub struct InputODLine {
    pub line_id: usize,
    pub weight: u32,
    pub start: Point,
    pub end: Point,
}
