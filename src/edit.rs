use crate::point::Point;

// Edit describes a change applied to a canvas.
#[derive(Debug)]
pub enum Edit {
    Right { start: Point, chars: Vec<char> },
    Down { start: Point, chars: Vec<char> },
}
