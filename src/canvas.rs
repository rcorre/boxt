use crate::{edit::Edit, point::Point, rect::Rect};

const EMPTY: char = ' ';

#[derive(Default, Debug, Clone)]
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

    fn get(&self, point: Point) -> char {
        self.current[point.y as usize][point.x as usize]
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
                        old.push(c);
                    }
                    undo.push(Edit::Right { start, chars: old });
                }
                Edit::Down { start, chars } => {
                    for (i, c) in chars.iter().enumerate() {
                        let c = self.put(start.x, start.y + i as u16, *c);
                        old.push(c);
                    }
                    undo.push(Edit::Down { start, chars: old });
                }
            }
        }

        // edits must be performed in the reverse order to undo
        undo.reverse();
        UndoRedo {
            edits: undo,
            size_x,
            size_y,
        }
    }

    pub fn edit(&mut self, edits: impl Iterator<Item = Edit>) {
        let undo = self.apply_edits(edits, true);
        log::debug!("Pushing undo: {undo:?}");
        self.undo.push(undo);
        self.redo.clear();
    }

    pub fn undo(&mut self) {
        let Some(undo) = self.undo.pop() else {
            log::info!("Nothing left to undo");
            return;
        };

        log::debug!("Performing undo: {undo:?}");
        let redo = self.apply_edits(undo.edits.into_iter(), false);

        log::debug!("Pushing redo: {redo:?}");
        self.redo.push(redo);

        // resize after, as an undo will typically shrink the canvas
        // if we shrink first, our edits will be out of bounds
        self.resize_y(undo.size_y, undo.size_x);
        self.resize_x(undo.size_x);
    }

    pub fn redo(&mut self) {
        let Some(redo) = self.redo.pop() else {
            log::info!("Nothing left to redo");
            return;
        };

        log::debug!("Performing redo: {redo:?}");

        // resize after, as an redo will typically expand the canvas
        // we need the canvas large enough to accomodate our edits
        self.resize_y(redo.size_y, redo.size_x);
        self.resize_x(redo.size_x);
        let undo = self.apply_edits(redo.edits.into_iter(), false);

        log::debug!("Pushing undo: {undo:?}");
        self.undo.push(undo);
    }

    pub fn clear(&mut self, point: Point) {
        self.edit(std::iter::once(Edit::Right {
            start: point,
            chars: vec![EMPTY],
        }));
    }

    fn find(&self, mut point: Point, dx: i16, dy: i16, c: &[char]) -> Option<Point> {
        let (size_y, size_x) = self.size();
        while point.x < size_x as u16 && point.y < size_y as u16 {
            if c.contains(&self.get(point)) {
                return Some(point);
            }
            point.x = if let Some(x) = point.x.checked_add_signed(dx) {
                x
            } else {
                return None;
            };
            point.y = if let Some(y) = point.y.checked_add_signed(dy) {
                y
            } else {
                return None;
            };
        }
        None
    }

    pub fn rect_around(&self, origin: Point) -> Option<Rect> {
        log::debug!("Finding rect around {origin:?}");
        let horizontal = &[
            Rect::HORIZONTAL,
            Rect::TOP_LEFT,
            Rect::TOP_RIGHT,
            Rect::BOTTOM_LEFT,
            Rect::BOTTOM_RIGHT,
        ];
        let vertical = &[
            Rect::VERTICAL,
            Rect::TOP_LEFT,
            Rect::TOP_RIGHT,
            Rect::BOTTOM_LEFT,
            Rect::BOTTOM_RIGHT,
        ];

        let Some(top) = self.find(origin, 0, -1, horizontal) else {
            log::debug!("No '{horizontal:?}' found above {origin:?}");
            return None;
        };
        let Some(bottom) = self.find(origin, 0, 1, horizontal) else {
            log::debug!("No '{horizontal:?}' found below {origin:?}");
            return None;
        };
        let Some(left) = self.find(origin, -1, 0, vertical) else {
            log::debug!("No '{vertical:?}' found left of {origin:?}");
            return None;
        };
        let Some(right) = self.find(origin, 1, 0, vertical) else {
            log::debug!("No '{vertical:?}' found right of {origin:?}");
            return None;
        };

        let top_left = Point {
            x: left.x,
            y: top.y,
        };
        let top_right = Point {
            x: right.x,
            y: top.y,
        };
        let bottom_left = Point {
            x: left.x,
            y: bottom.y,
        };
        let bottom_right = Point {
            x: right.x,
            y: bottom.y,
        };

        if self.get(top_left) != Rect::TOP_LEFT {
            log::debug!("No rect corner found at {top_left:?}");
            return None;
        }
        if self.get(top_right) != Rect::TOP_RIGHT {
            log::debug!("No rect corner found at {top_right:?}");
            return None;
        }
        if self.get(bottom_left) != Rect::BOTTOM_LEFT {
            log::debug!("No rect corner found at {bottom_left:?}");
            return None;
        }
        if self.get(bottom_right) != Rect::BOTTOM_RIGHT {
            log::debug!("No rect corner found at {bottom_right:?}");
            return None;
        }

        Some(Rect {
            top_left,
            bottom_right,
        })
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
    use crate::{point::Point, rect::Rect};
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
    fn test_canvas_undo_redo() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut c = Canvas::new(4, 4);

        let state0 = c.to_string();
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

        c.undo();
        assert_eq!(c.to_string(), state0);

        c.redo();
        assert_eq!(c.to_string(), state1);

        c.redo();
        assert_eq!(c.to_string(), state2);
    }

    #[test]
    fn test_canvas_edit_clears_redo() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut c = Canvas::new(4, 4);

        let state0 = c.to_string();
        let state1 = "      
  ---+
     |
     |";
        let state2 = "       
  +    
  | ---
       ";

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

        c.undo();
        assert_eq!(c.to_string(), state0);

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

        c.redo();
        assert_eq!(c.to_string(), state2);

        c.undo();
        assert_eq!(c.to_string(), state0);

        c.redo();
        assert_eq!(c.to_string(), state2);
    }

    #[test]
    fn test_match_rect() {
        let mut c = Canvas::new(16, 8);
        let expected = Rect::new(3, 2, 8, 5);
        c.edit(expected.edits().into_iter());

        // BUG: Selecting on the borders does not select the correct rect bounds
        for y in 0..7 {
            for x in 0..12 {
                let point = Point { x, y };
                let expected = if x > 3 && x < 8 && y > 2 && y < 5 {
                    Some(expected)
                } else {
                    None
                };
                assert_eq!(c.rect_around(point), expected, "{point:?}");
            }
        }
    }
}
