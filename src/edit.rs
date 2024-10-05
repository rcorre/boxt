use crate::point::Point;

// Describes the direction of an edit and it's length.
// RIGHT(4) means 4 characters to the right of start.
#[derive(Debug)]
pub enum Direction {
    Point,
    Right(u16),
    Down(u16),
}

// Edit describes a change applied to a canvas.
// For example, {start: (1,2), char: '-', dir: Right(4)}
// means to insert '-' from (1,2) through (4,2) (not including (5,2))
#[derive(Debug)]
pub struct Edit {
    pub start: Point,
    pub char: char,
    pub dir: Direction,
}
