const EMPTY: &str = " ";

#[derive(Default, Clone)]
pub struct Canvas(Vec<Vec<String>>);

impl Canvas {
    pub fn new(size_x: u16, size_y: u16) -> Canvas {
        Self(vec![vec![EMPTY.into(); size_x as usize]; size_y as usize])
    }

    // Set a cell to a string. Expands to accomodate the cell if needed.
    pub fn put(&mut self, x: u16, y: u16, s: &str) {
        let x = x as usize;
        let y = y as usize;
        let size_y = self.0.len();
        let size_x = self.0.first().map(|r| r.len()).unwrap_or(0);
        let new_size_y = std::cmp::max(size_y, y + 1);
        let new_size_x = std::cmp::max(size_x, x + 1);
        if new_size_y >= size_y {
            self.0.resize(new_size_y, vec![EMPTY.into(); new_size_x]);
        }
        if new_size_x >= size_x {
            for row in &mut self.0 {
                row.resize(new_size_x, EMPTY.into());
            }
        }

        self.0[y][x] = s.into();
    }

    pub fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|row| row.join(""))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
