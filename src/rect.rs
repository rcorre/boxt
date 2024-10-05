use crate::canvas::Canvas;
use crate::edit::Edit;
use crate::point::Point;

#[derive(Debug)]
pub struct Rect {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl Rect {
    // ┌───┐
    // │   │
    // └───┘

    pub fn new(x1: u16, y1: u16, x2: u16, y2: u16) -> Rect {
        Self {
            top_left: Point { x: x1, y: y1 },
            bottom_right: Point { x: x2, y: y2 },
        }
    }

    pub fn edits(&self) -> Vec<Edit> {
        let Rect {
            top_left: Point { x: x1, y: y1 },
            bottom_right: Point { x: x2, y: y2 },
        } = *self;

        let (x1, x2) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        let (y1, y2) = if y1 < y2 { (y1, y2) } else { (y2, y1) };

        let top_left = Point { x: x1, y: y1 };
        let bottom_left = Point { x: x1, y: y2 };
        let top_right = Point { x: x2, y: y1 };
        let w = (x2 - x1) as usize;
        let h = (y2 - y1) as usize;

        const TOP_LEFT: char = '+';
        const TOP_RIGHT: char = '+';
        const HORIZONTAL: char = '-';
        const VERTICAL: char = '|';
        const BOTTOM_LEFT: char = '+';
        const BOTTOM_RIGHT: char = '+';

        let mut top = vec![HORIZONTAL; w + 1];
        top[0] = TOP_LEFT;
        top[w] = TOP_RIGHT;

        let mut bottom = vec![HORIZONTAL; w + 1];
        bottom[0] = BOTTOM_LEFT;
        bottom[w] = BOTTOM_RIGHT;

        let side = vec![VERTICAL; h];

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

    pub fn draw(&self, canvas: &mut Canvas) {
        let Rect {
            top_left: Point { x: x1, y: y1 },
            bottom_right: Point { x: x2, y: y2 },
        } = *self;

        if x1 == x2 || y1 == y2 {
            log::warn!("Zero-area rect");
            return;
        }

        let (x1, x2) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        let (y1, y2) = if y1 < y2 { (y1, y2) } else { (y2, y1) };

        const TOP_LEFT: char = '+';
        const TOP_RIGHT: char = '+';
        const HORIZONTAL: char = '-';
        const VERTICAL: char = '|';
        const BOTTOM_LEFT: char = '+';
        const BOTTOM_RIGHT: char = '+';

        for y in y1..y2 {
            canvas.put(x1, y, VERTICAL);
            canvas.put(x2, y, VERTICAL);
        }

        for x in x1..x2 {
            canvas.put(x, y1, HORIZONTAL);
            canvas.put(x, y2, HORIZONTAL);
        }

        canvas.put(x1, y1, TOP_LEFT);
        canvas.put(x2, y1, TOP_RIGHT);
        canvas.put(x1, y2, BOTTOM_LEFT);
        canvas.put(x2, y2, BOTTOM_RIGHT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
