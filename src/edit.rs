use crate::point::Point;

// Edit describes a change applied to a canvas.
#[derive(Debug, Clone)]
pub enum Edit {
    Right { start: Point, chars: Vec<char> },
    Down { start: Point, chars: Vec<char> },
}

impl Edit {
    // The canvas size required to accomodate this edit.
    pub fn bounds(&self) -> Point {
        match self {
            Edit::Right { chars, .. } | Edit::Down { chars, .. } if chars.is_empty() => {
                Point { x: 0, y: 0 }
            }
            Edit::Right { start, chars } => Point {
                x: start.x + chars.len() as u16,
                y: start.y + 1,
            },
            Edit::Down { start, chars } => Point {
                x: start.x + 1,
                y: start.y + chars.len() as u16,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_edit_bounds_empty() {
        let actual = Edit::Right {
            start: Point { x: 1, y: 2 },
            chars: vec![],
        };
        assert_eq!(actual.bounds(), Point { x: 0, y: 0 });

        let actual = Edit::Down {
            start: Point { x: 4, y: 3 },
            chars: vec![],
        };
        assert_eq!(actual.bounds(), Point { x: 0, y: 0 });
    }

    #[test]
    fn test_edit_bounds_right() {
        let actual = Edit::Right {
            start: Point { x: 1, y: 2 },
            chars: vec!['a', 'b', 'c'],
        };
        assert_eq!(actual.bounds(), Point { x: 4, y: 3 });
    }

    #[test]
    fn test_edit_bounds_down() {
        let actual = Edit::Down {
            start: Point { x: 4, y: 3 },
            chars: vec!['a', 'b', 'c'],
        };
        assert_eq!(actual.bounds(), Point { x: 5, y: 6 });
    }
}
