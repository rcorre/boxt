use crate::vec::UVec;

// Edit describes a change applied to a canvas.
#[derive(Debug, Clone)]
pub enum Edit {
    Right { start: UVec, chars: Vec<char> },
    Down { start: UVec, chars: Vec<char> },
}

impl Edit {
    // The canvas size required to accomodate this edit.
    pub fn bounds(&self) -> UVec {
        match self {
            Edit::Right { chars, .. } | Edit::Down { chars, .. } if chars.is_empty() => {
                UVec { x: 0, y: 0 }
            }
            Edit::Right { start, chars } => UVec {
                x: start.x + chars.len() as u16,
                y: start.y + 1,
            },
            Edit::Down { start, chars } => UVec {
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
            start: UVec { x: 1, y: 2 },
            chars: vec![],
        };
        assert_eq!(actual.bounds(), UVec { x: 0, y: 0 });

        let actual = Edit::Down {
            start: UVec { x: 4, y: 3 },
            chars: vec![],
        };
        assert_eq!(actual.bounds(), UVec { x: 0, y: 0 });
    }

    #[test]
    fn test_edit_bounds_right() {
        let actual = Edit::Right {
            start: UVec { x: 1, y: 2 },
            chars: vec!['a', 'b', 'c'],
        };
        assert_eq!(actual.bounds(), UVec { x: 4, y: 3 });
    }

    #[test]
    fn test_edit_bounds_down() {
        let actual = Edit::Down {
            start: UVec { x: 4, y: 3 },
            chars: vec!['a', 'b', 'c'],
        };
        assert_eq!(actual.bounds(), UVec { x: 5, y: 6 });
    }
}
