use crate::edit::Edit;
use crate::vec::UVec;

#[derive(Debug)]
pub struct Line {
    pub start: UVec,
    pub end: UVec,
    pub mirror: bool,
}

impl Line {
    pub const HORIZONTAL: char = '-';
    pub const VERTICAL: char = '|';
    pub const CORNER: char = '+';

    pub fn new(start: UVec, end: UVec) -> Self {
        Self {
            start,
            end,
            mirror: false,
        }
    }

    fn line(char: char, len: usize) -> Vec<char> {
        let mut chars = vec![char; len + 1];
        chars[0] = Self::CORNER;
        chars[len] = Self::CORNER;
        chars
    }

    fn vert(a: UVec, b: UVec) -> Edit {
        let dy = b.y.abs_diff(a.y) as usize;
        Edit::Down {
            start: UVec {
                x: a.x,
                y: std::cmp::min(a.y, b.y),
            },
            chars: Self::line(Self::VERTICAL, dy),
        }
    }

    fn horiz(a: UVec, b: UVec) -> Edit {
        let dx = b.x.abs_diff(a.x) as usize;
        Edit::Right {
            start: UVec {
                x: std::cmp::min(a.x, b.x),
                y: a.y,
            },
            chars: Self::line(Self::HORIZONTAL, dx),
        }
    }

    pub fn edits(&self) -> Vec<Edit> {
        let (a, b) = (self.start, self.end);

        if self.mirror {
            vec![Self::horiz(a, b), Self::vert(UVec { y: a.y, x: b.x }, b)]
        } else {
            vec![Self::vert(a, b), Self::horiz(UVec { x: a.x, y: b.y }, b)]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::canvas::Canvas;

    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_draw_line_one_point() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line::new(UVec { x: 1, y: 1 }, UVec { x: 1, y: 1 });
        canvas.edit(r.edits().into_iter());
        assert_eq!(canvas.to_string().trim(), "+")
    }

    #[test]
    fn test_draw_line_down_right() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line::new(UVec { x: 1, y: 1 }, UVec { x: 4, y: 3 });
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_up_right() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line::new(UVec { x: 1, y: 3 }, UVec { x: 4, y: 1 });
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_up_left() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line::new(UVec { x: 4, y: 3 }, UVec { x: 1, y: 1 });
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_down_left() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line::new(UVec { x: 4, y: 1 }, UVec { x: 1, y: 3 });
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_down_right_mirror() {
        let mut canvas = Canvas::new(8, 8);
        let mut r = Line::new(UVec { x: 1, y: 1 }, UVec { x: 4, y: 3 });
        r.mirror = true;
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_up_right_mirror() {
        let mut canvas = Canvas::new(8, 8);
        let mut r = Line::new(UVec { x: 1, y: 3 }, UVec { x: 4, y: 1 });
        r.mirror = true;
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_up_left_mirror() {
        let mut canvas = Canvas::new(8, 8);
        let mut r = Line::new(UVec { x: 4, y: 3 }, UVec { x: 1, y: 1 });
        r.mirror = true;
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_down_left_mirror() {
        let mut canvas = Canvas::new(8, 8);
        let mut r = Line::new(UVec { x: 4, y: 1 }, UVec { x: 1, y: 3 });
        r.mirror = true;
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }
}
