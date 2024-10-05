use crate::canvas::Canvas;
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

    pub fn draw(&self, canvas: &mut Canvas) {
        for (i, c) in self.text.chars().enumerate() {
            canvas.put(self.start.x + i as u16, self.start.y, &c.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_text_empty() {
        let mut canvas = Canvas::new(8, 8);
        let t = Text::new(0, 0, "");
        t.draw(&mut canvas);
        assert_eq!(canvas.to_string().trim(), "")
    }

    #[test]
    fn test_draw_text_0011() {
        let mut canvas = Canvas::new(2, 2);
        let t = Text::new(2, 1, "foo");
        t.draw(&mut canvas);
        assert_eq!(
            canvas.to_string().trim(),
            "\
  foo
  "
        )
    }
}
