#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub struct UVec {
    pub x: u16,
    pub y: u16,
}

impl UVec {
    // Stops at 0.
    pub fn translated(&self, d: IVec) -> Self {
        Self {
            x: self.x.saturating_add_signed(d.x),
            y: self.y.saturating_add_signed(d.y),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub struct IVec {
    pub x: i16,
    pub y: i16,
}

impl IVec {
    pub const UP: IVec = IVec { x: 0, y: -1 };
    pub const DOWN: IVec = IVec { x: 0, y: 1 };
    pub const LEFT: IVec = IVec { x: -1, y: 0 };
    pub const RIGHT: IVec = IVec { x: 1, y: 0 };
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_uvec_translate() {
        let p = UVec { x: 2, y: 3 };
        assert_eq!(p.translated(IVec { x: 0, y: 0 }), UVec { x: 2, y: 3 });
        assert_eq!(p.translated(IVec { x: 1, y: 0 }), UVec { x: 3, y: 3 });
        assert_eq!(p.translated(IVec { x: 0, y: 1 }), UVec { x: 2, y: 4 });
        assert_eq!(p.translated(IVec { x: -1, y: 0 }), UVec { x: 1, y: 3 });
        assert_eq!(p.translated(IVec { x: 0, y: -1 }), UVec { x: 2, y: 2 });
        assert_eq!(p.translated(IVec { x: -5, y: -10 }), UVec { x: 0, y: 0 });
    }
}
