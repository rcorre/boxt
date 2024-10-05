#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
        }
    }

    pub fn down(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }
}
