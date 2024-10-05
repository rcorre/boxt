use crate::edit::Edit;
use crate::point::Point;

#[derive(Debug)]
pub struct Line(pub Vec<Point>);

impl Line {
    pub fn edits(&self) -> Vec<Edit> {
        const HORIZONTAL: char = '-';
        const VERTICAL: char = '|';
        const CORNER: char = '+';

        let mut edits = vec![];

        for [a, b] in self.0.array_windows() {
            // Start with the uppermost point and go down.
            let (a, b) = if b.y > a.y { (a, b) } else { (b, a) };

            let dy = (b.y - a.y) as usize;
            let mut chars = vec![VERTICAL; dy + 1];
            chars[0] = CORNER;
            chars[dy] = CORNER;
            edits.push(Edit::Down {
                start: a.clone(),
                chars,
            });

            // Now move horizontally
            let dx = b.x.abs_diff(a.x) as usize;
            let mut chars = vec![HORIZONTAL; dx + 1];
            chars[0] = CORNER;
            chars[dx] = CORNER;

            edits.push(Edit::Right {
                start: Point {
                    x: std::cmp::min(a.x, b.x),
                    y: b.y,
                },
                chars,
            });
        }

        edits
    }
}

#[cfg(test)]
mod tests {
    use crate::canvas::Canvas;

    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_draw_line_empty() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![]);
        canvas.edit(r.edits().into_iter());
        assert_eq!(canvas.to_string().trim(), "")
    }

    #[test]
    fn test_draw_line_one_point() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 1, y: 1 }]);
        canvas.edit(r.edits().into_iter());
        assert_eq!(canvas.to_string().trim(), "")
    }

    #[test]
    fn test_draw_line_down_right() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 1, y: 1 }, Point { x: 4, y: 3 }]);
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_up_right() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 1, y: 3 }, Point { x: 4, y: 1 }]);
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_up_left() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 4, y: 3 }, Point { x: 1, y: 1 }]);
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }

    #[test]
    fn test_draw_line_down_left() {
        let mut canvas = Canvas::new(8, 8);
        let r = Line(vec![Point { x: 4, y: 1 }, Point { x: 1, y: 3 }]);
        canvas.edit(r.edits().into_iter());
        assert_snapshot!(canvas.to_string())
    }
}
