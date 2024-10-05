pub mod canvas;
pub mod tui;

use canvas::Canvas;

// TODO: use u16, maybe use x1/x2
#[derive(Debug)]
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

        const TOP_LEFT: &str = "┌";
        const TOP_RIGHT: &str = "┐";
        const HORIZONTAL: &str = "─";
        const VERTICAL: &str = "│";
        const BOTTOM_LEFT: &str = "└";
        const BOTTOM_RIGHT: &str = "┘";

        let top = self.y;
        let bottom = self.y + self.h - 1;
        let left = self.x;
        let right = self.x + self.w - 1;

        for y in top..bottom {
            canvas.put(left, y, VERTICAL);
            canvas.put(right, y, VERTICAL);
        }

        for x in left..right {
            canvas.put(x, top, HORIZONTAL);
            canvas.put(x, bottom, HORIZONTAL);
        }

        canvas.put(left, top, TOP_LEFT);
        canvas.put(right, top, TOP_RIGHT);
        canvas.put(left, bottom, BOTTOM_LEFT);
        canvas.put(right, bottom, BOTTOM_RIGHT);
    }
}
