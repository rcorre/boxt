use crate::edit::Edit;
use crate::point::Point;

#[derive(Debug)]
pub struct Text {
    pub start: Point,
    pub text: String,
}

impl Text {
    pub fn new(x: u16, y: u16, text: &str) -> Text {
        Self {
            start: Point { x, y },
            text: text.into(),
        }
    }

    pub fn edits(&self) -> Vec<Edit> {
        self.text
            .lines()
            .enumerate()
            .map(|(i, line)| Edit::Right {
                start: Point {
                    x: self.start.x,
                    y: self.start.y + i as u16,
                },
                chars: line.chars().collect(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::canvas::Canvas;

    use super::*;

    #[test]
    fn test_draw_text_empty() {
        let mut canvas = Canvas::new(8, 8);
        let t = Text::new(0, 0, "");
        canvas.edit(t.edits().into_iter());
        assert_eq!(canvas.to_string().trim(), "")
    }

    #[test]
    fn test_draw_text() {
        let mut canvas = Canvas::new(2, 2);
        let t = Text::new(2, 1, "foo");
        canvas.edit(t.edits().into_iter());
        assert_eq!(
            canvas.to_string().trim(),
            "\
  foo"
        )
    }

    #[test]
    fn test_draw_text_multiline() {
        let mut canvas = Canvas::new(2, 2);
        let t = Text::new(2, 1, "foo\nbar\nbaz");
        canvas.edit(t.edits().into_iter());
        assert_eq!(
            canvas.to_string().trim(),
            "\
  foo
  bar
  baz"
        )
    }
}
