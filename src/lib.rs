pub mod canvas;
pub mod tui;

use canvas::Canvas;

// TODO: use u16, maybe use x1/x2
#[derive(Debug)]
struct Rect {
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
}

impl Rect {
    // ┌───┐
    // │   │
    // └───┘

    pub fn draw(&self, canvas: &mut Canvas) {
        let Rect { x1, y1, x2, y2 } = *self;

        if x1 == x2 || y1 == y2 {
            log::warn!("Zero-area rect");
            return;
        }

        const TOP_LEFT: &str = "+";
        const TOP_RIGHT: &str = "+";
        const HORIZONTAL: &str = "-";
        const VERTICAL: &str = "|";
        const BOTTOM_LEFT: &str = "+";
        const BOTTOM_RIGHT: &str = "+";

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
        let r = Rect {
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
        };
        r.draw(&mut canvas);
        assert_eq!(canvas.to_string().trim(), "")
    }

    #[test]
    fn test_draw_rect_0011() {
        let mut canvas = Canvas::new(2, 2);
        let r = Rect {
            x1: 0,
            y1: 0,
            x2: 1,
            y2: 1,
        };
        r.draw(&mut canvas);
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
        let r = Rect {
            x1: 0,
            y1: 0,
            x2: 4,
            y2: 2,
        };
        r.draw(&mut canvas);
        assert_eq!(
            canvas.to_string().trim(),
            "\
+---+
|   |
+---+"
        )
    }

    //     #[test]
    //     fn test_draw_rect_3154() {
    //         let mut canvas = Canvas::new(5, 3);
    //         let r = Rect {
    //             x1: 3,
    //             y1: 1,
    //             x2: 5,
    //             y2: 4,
    //         };
    //         r.draw(&mut canvas);
    //         assert_eq!(
    //             canvas.to_string(),
    //             "    \
    // \n
    //    +-+
    //    | |
    //    | |
    //    +-+"
    //         )
    //     }
}
