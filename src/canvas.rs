use crate::{edit::Edit, point::Point};

const EMPTY: char = ' ';

#[derive(Default, Clone)]
struct UndoRedo {
    edits: Vec<Edit>,
    size_x: usize,
    size_y: usize,
}

#[derive(Default, Clone)]
pub struct Canvas {
    current: Vec<Vec<char>>,
    undo: Vec<UndoRedo>,
    redo: Vec<UndoRedo>,
}

impl Canvas {
    pub fn new(size_x: u16, size_y: u16) -> Canvas {
        Self {
            current: vec![vec![EMPTY; size_x as usize]; size_y as usize],
            ..Default::default()
        }
    }

    fn resize_y(&mut self, size_y: usize, size_x: usize) {
        log::debug!("Resizing y to {size_y}x{size_x}");
        self.current.resize(size_y, vec![EMPTY.into(); size_x]);
    }

    fn resize_x(&mut self, size: usize) {
        log::debug!("Resizing x to {size}");
        for row in &mut self.current {
            row.resize(size, EMPTY.into());
        }
    }

    pub fn from_str(s: &str) -> Canvas {
        let w = s.lines().map(|l| l.len()).sum();
        let current = s
            .lines()
            .map(|l| {
                let mut v: Vec<char> = l.chars().collect();
                v.resize(w, EMPTY);
                v
            })
            .collect();
        Self {
            current,
            ..Default::default()
        }
    }

    // Returns the list of edits to undo this edit.
    fn apply_edits(&mut self, edits: impl Iterator<Item = Edit>, expand: bool) -> UndoRedo {
        let (size_y, size_x) = self.size();
        let mut undo = vec![];
        for e in edits {
            log::trace!("Applying edit: {e:?}");
            let mut old = vec![];
            if expand {
                self.maybe_expand(e.bounds());
            }
            match e {
                Edit::Right { start, chars } => {
                    for (i, c) in chars.iter().enumerate() {
                        let c = self.put(start.x + i as u16, start.y, *c);
                        if start.x as usize + i < size_x && (start.y as usize) < size_y {
                            // only create an undo entry for things that fit in the original canvas
                            old.push(c);
                        }
                    }
                    undo.push(Edit::Right { start, chars: old });
                }
                Edit::Down { start, chars } => {
                    for (i, c) in chars.iter().enumerate() {
                        let c = self.put(start.x, start.y + i as u16, *c);
                        if (start.x as usize) < size_x && (start.y as usize + i) < size_y {
                            // only create an undo entry for things that fit in the original canvas
                            old.push(c);
                        }
                    }
                    undo.push(Edit::Down { start, chars: old });
                }
            }
        }
        UndoRedo {
            edits: undo,
            size_x,
            size_y,
        }
    }

    pub fn edit(&mut self, edits: impl Iterator<Item = Edit>) {
        let undo = self.apply_edits(edits, true);
        self.undo.push(undo);
    }

    pub fn undo(&mut self) {
        let Some(UndoRedo {
            edits,
            size_x,
            size_y,
        }) = self.undo.pop()
        else {
            log::info!("Nothing left to undo");
            return;
        };

        log::info!("Performing undo");

        self.resize_y(size_y, size_x);
        self.resize_x(size_x);
        let redo = self.apply_edits(edits.into_iter(), false);
        self.redo.push(redo);
    }

    pub fn clear(&mut self, point: Point) {
        self.edit(std::iter::once(Edit::Right {
            start: point,
            chars: vec![EMPTY],
        }));
    }

    // Returns (size_y, size_x).
    fn size(&self) -> (usize, usize) {
        (
            self.current.len(),
            self.current.first().map(|r| r.len()).unwrap_or(0),
        )
    }

    fn maybe_expand(&mut self, bounds: Point) {
        let (size_y, size_x) = self.size();
        let new_size_y = std::cmp::max(size_y, bounds.y as usize);
        let new_size_x = std::cmp::max(size_x, bounds.x as usize);
        if new_size_y > size_y {
            self.resize_y(new_size_y, new_size_x);
        }
        if new_size_x > size_x {
            self.resize_x(new_size_x);
        }
    }

    fn put(&mut self, x: u16, y: u16, c: char) -> char {
        log::trace!("Putting {c} at {x},{y}");
        std::mem::replace(&mut self.current[y as usize][x as usize], c)
    }

    pub fn to_string(&self) -> String {
        self.current
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point::Point;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_canvas_edit() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut c = Canvas::new(4, 4);
        c.edit(
            vec![
                Edit::Right {
                    start: Point { x: 2, y: 1 },
                    chars: vec!['-', '-', '-', '+'],
                },
                Edit::Down {
                    start: Point { x: 5, y: 2 },
                    chars: vec!['|', '|'],
                },
            ]
            .into_iter(),
        );

        assert_eq!(
            c.to_string(),
            "      
  ---+
     |
     |"
        );
    }

    #[test]
    fn test_canvas_undo() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut c = Canvas::new(4, 4);

        let state0 = "";
        let state1 = "      
  ---+
     |
     |";
        let state2 = "       
  +--+ 
  | ---
     | ";

        c.edit(
            vec![
                Edit::Right {
                    start: Point { x: 2, y: 1 },
                    chars: vec!['-', '-', '-', '+'],
                },
                Edit::Down {
                    start: Point { x: 5, y: 2 },
                    chars: vec!['|', '|'],
                },
            ]
            .into_iter(),
        );
        assert_eq!(c.to_string(), state1);

        c.edit(
            vec![
                Edit::Down {
                    start: Point { x: 2, y: 1 },
                    chars: vec!['+', '|'],
                },
                Edit::Right {
                    start: Point { x: 4, y: 2 },
                    chars: vec!['-', '-', '-'],
                },
            ]
            .into_iter(),
        );
        assert_eq!(c.to_string(), state2);

        c.undo();
        assert_eq!(c.to_string(), state1);

        // c.undo();
        // assert_eq!(c.to_string(), state0);
    }
}
