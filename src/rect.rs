use crate::edit::Edit;
use crate::vec::UVec;

#[derive(Copy, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Rect {
    pub top_left: UVec,
    pub bottom_right: UVec,
}

impl Rect {
    // ┌───┐
    // │   │
    // └───┘

    pub const TOP_LEFT: char = '+';
    pub const TOP_RIGHT: char = '+';
    pub const HORIZONTAL: char = '-';
    pub const VERTICAL: char = '|';
    pub const BOTTOM_LEFT: char = '+';
    pub const BOTTOM_RIGHT: char = '+';
    pub const CORNERS: [char; 4] = [
        Self::TOP_LEFT,
        Self::TOP_RIGHT,
        Self::BOTTOM_LEFT,
        Self::BOTTOM_RIGHT,
    ];

    pub fn new(x1: u16, y1: u16, x2: u16, y2: u16) -> Rect {
        Self {
            top_left: UVec { x: x1, y: y1 },
            bottom_right: UVec { x: x2, y: y2 },
        }
    }

    pub fn edits(&self) -> Vec<Edit> {
        let Rect {
            top_left: UVec { x: x1, y: y1 },
            bottom_right: UVec { x: x2, y: y2 },
        } = *self;

        let (x1, x2) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        let (y1, y2) = if y1 < y2 { (y1, y2) } else { (y2, y1) };

        let top_left = UVec { x: x1, y: y1 };
        let bottom_left = UVec { x: x1, y: y2 };
        let top_right = UVec { x: x2, y: y1 };
        let w = (x2 - x1) as usize;
        let h = (y2 - y1) as usize;

        let mut top = vec![Self::HORIZONTAL; w + 1];
        top[0] = Self::TOP_LEFT;
        top[w] = Self::TOP_RIGHT;

        let mut bottom = vec![Self::HORIZONTAL; w + 1];
        bottom[0] = Self::BOTTOM_LEFT;
        bottom[w] = Self::BOTTOM_RIGHT;

        let side = vec![Self::VERTICAL; h.saturating_sub(1)];

        vec![
            Edit::Right {
                start: top_left,
                chars: top,
            },
            Edit::Right {
                start: bottom_left,
                chars: bottom,
            },
            Edit::Down {
                start: top_left.down(),
                chars: side.clone(),
            },
            Edit::Down {
                start: top_right.down(),
                chars: side,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::canvas::Canvas;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_draw_rect_0000() {
        let mut canvas = Canvas::new(8, 8);
        let r = Rect::new(0, 0, 0, 0);
        canvas.edit(r.edits().into_iter());
        assert_eq!(canvas.to_string().trim(), "+")
    }

    #[test]
    fn test_draw_rect_0011() {
        let mut canvas = Canvas::new(2, 2);
        let r = Rect::new(0, 0, 1, 1);
        canvas.edit(r.edits().into_iter());
        assert_eq!(
            canvas.to_string().trim(),
            "\
++
++"
        )
    }

    #[test]
    fn test_draw_rect_0042() {
        let mut canvas = Canvas::new(5, 3);
        let r = Rect::new(0, 0, 4, 2);
        canvas.edit(r.edits().into_iter());
        assert_eq!(
            canvas.to_string().trim(),
            "\
+---+
|   |
+---+"
        )
    }

    #[test]
    fn test_draw_rect_4200() {
        let mut canvas = Canvas::new(5, 3);
        let r = Rect::new(4, 2, 0, 0);
        canvas.edit(r.edits().into_iter());
        assert_eq!(
            canvas.to_string().trim(),
            "\
+---+
|   |
+---+"
        )
    }

    #[test]
    fn test_draw_rect_0240() {
        let mut canvas = Canvas::new(5, 3);
        let r = Rect::new(0, 2, 4, 0);
        canvas.edit(r.edits().into_iter());
        assert_eq!(
            canvas.to_string().trim(),
            "\
+---+
|   |
+---+"
        )
    }

    #[test]
    fn test_draw_rect_4002() {
        let mut canvas = Canvas::new(5, 3);
        let r = Rect::new(4, 0, 0, 2);
        canvas.edit(r.edits().into_iter());
        assert_eq!(
            canvas.to_string().trim(),
            "\
+---+
|   |
+---+"
        )
    }
}
