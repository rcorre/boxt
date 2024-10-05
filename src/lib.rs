pub mod tui;

use std::io::Write;

use anyhow::Result;
use serde::{Deserialize, Serialize};

type Canvas = Vec<Vec<String>>;

#[derive(Serialize, Deserialize)]
struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Rect {
    // ┌───┐
    // │   │
    // └───┘

    pub fn draw(&self, canvas: &mut Canvas) {
        if self.w == 0 || self.h == 0 {
            log::warn!("Zero-area rect");
            return;
        }

        const top_left: &str = "┌";
        const top_right: &str = "┐";
        const horizontal: &str = "─";
        const vertical: &str = "│";
        const bottom_left: &str = "└";
        const bottom_right: &str = "┘";

        let top = self.y;
        let bottom = self.y + self.h - 1;
        let left = self.x;
        let right = self.x + self.w - 1;

        for y in top..bottom {
            canvas[y][left] = vertical.into();
            canvas[y][right] = vertical.into();
        }

        for x in left..right {
            canvas[top][x] = horizontal.into();
            canvas[bottom][x] = horizontal.into();
        }

        canvas[top][left] = top_left.into();
        canvas[top][right] = top_right.into();
        canvas[bottom][left] = bottom_left.into();
        canvas[bottom][right] = bottom_right.into();
    }
}

#[derive(Serialize, Deserialize)]
pub struct Document {
    rect: Vec<Rect>,
}

impl Document {
    fn max_xy(&self) -> (usize, usize) {
        (
            self.rect.iter().map(|r| r.x + r.w).max().unwrap_or(0),
            self.rect.iter().map(|r| r.y + r.h).max().unwrap_or(0),
        )
    }

    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Document> {
        let s = std::fs::read_to_string(path)?;
        log::trace!("Parsing:\n{s}");
        let doc = toml::from_str(&s)?;
        Ok(doc)
    }

    pub fn to_string(&self) -> String {
        let (max_x, max_y) = self.max_xy();
        let mut res = vec![vec![" ".to_string(); max_x]; max_y];
        for r in &self.rect {
            r.draw(&mut res);
        }

        let mut s = String::new();
        for row in res {
            s += &row.join("");
            s += "\n";
        }

        s
    }

    pub fn draw(&self, w: &mut impl Write) -> Result<()> {
        let (max_x, max_y) = self.max_xy();
        let mut res = vec![vec![" ".to_string(); max_x]; max_y];
        for r in &self.rect {
            r.draw(&mut res);
        }

        for row in res {
            writeln!(w, "{}", row.join(""))?;
        }

        Ok(())
    }
}
