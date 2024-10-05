use crate::canvas::Canvas;
use crate::point::Point;

#[derive(Debug)]
struct Line(pub Vec<Point>);

impl Line {
    pub fn draw(&self, canvas: &mut Canvas) {
        const HORIZONTAL: &str = "-";
        const VERTICAL: &str = "|";
        const TOP_LEFT: &str = "+";
        const TOP_RIGHT: &str = "+";
        const BOTTOM_LEFT: &str = "+";
        const BOTTOM_RIGHT: &str = "+";

        for [a, b] in self.0.array_windows() {
            if b.y > a.y {
                for y in a.y..b.y {
                    canvas.put(a.x, y, VERTICAL);
                }
            } else {
                for y in b.y..a.y {
                    canvas.put(b.x, y, VERTICAL);
                }
            }

            let y = std::cmp::max(a.y, b.y);
            if b.x > a.x {
                for x in a.x..b.x {
                    canvas.put(x, y, HORIZONTAL);
                }
            } else {
                for x in b.x..a.x {
                    canvas.put(x, y, HORIZONTAL);
                }
            }

            canvas.put(a.x, a.y, TOP_LEFT);
            canvas.put(b.x, b.y, TOP_LEFT);
            canvas.put(std::cmp::min(a.x, b.x), std::cmp::max(a.y, b.y), TOP_LEFT);
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
