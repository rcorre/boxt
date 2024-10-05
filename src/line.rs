use crate::canvas::Canvas;
use crate::point::Point;

#[derive(Debug)]
pub struct Line(pub Vec<Point>);

impl Line {
    pub fn draw(&self, canvas: &mut Canvas) {
        const HORIZONTAL: char = '-';
        const VERTICAL: char = '|';
        const TOP_LEFT: char = '+';

        for [a, b] in self.0.array_windows() {
            let (a, b) = if b.y > a.y { (a, b) } else { (b, a) };
            for y in a.y..b.y {
                canvas.put(a.x, y, VERTICAL);
            }
            if a.x < b.x {
                for x in a.x..b.x {
                    canvas.put(x, b.y, HORIZONTAL);
                }
            } else {
                for x in b.x..a.x {
                    canvas.put(x, b.y, HORIZONTAL);
                }
            }

            canvas.put(a.x, a.y, TOP_LEFT);
            canvas.put(a.x, b.y, TOP_LEFT);
            canvas.put(b.x, b.y, TOP_LEFT);
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_draw_line_empty() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![]);
        r.draw(&mut canvas);
        assert_eq!(canvas.to_string().trim(), "")
    }

    #[test]
    fn test_draw_line_one_point() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 1, y: 1 }]);
        r.draw(&mut canvas);
        assert_eq!(canvas.to_string().trim(), "")
    }

    #[test]
    fn test_draw_line_down_right() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 1, y: 1 }, Point { x: 4, y: 3 }]);
        r.draw(&mut canvas);
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_up_right() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 1, y: 3 }, Point { x: 4, y: 1 }]);
        r.draw(&mut canvas);
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_up_left() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 4, y: 3 }, Point { x: 1, y: 1 }]);
        r.draw(&mut canvas);
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_down_left() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 4, y: 1 }, Point { x: 1, y: 3 }]);
        r.draw(&mut canvas);
        assert_snapshot!(canvas.to_string())
    }
}
