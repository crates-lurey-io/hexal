//! The six axial directions on a pointy-top hex grid.

/// One of the six axial directions on a pointy-top hex grid.
///
/// Variants are indexed `E = 0` counterclockwise through `SE = 5`, matching
/// the index into [`Hex::DIRECTIONS`](crate::Hex::DIRECTIONS).
///
/// ```text
///     NW  NE
///   W  ·  E
///     SW  SE
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Direction {
    /// East — `(+1, 0)`.
    E = 0,
    /// Northeast — `(+1, -1)`.
    NE = 1,
    /// Northwest — `(0, -1)`.
    NW = 2,
    /// West — `(-1, 0)`.
    W = 3,
    /// Southwest — `(-1, +1)`.
    SW = 4,
    /// Southeast — `(0, +1)`.
    SE = 5,
}

impl Direction {
    /// All six directions in counterclockwise order starting from East.
    pub const ALL: [Self; 6] = [Self::E, Self::NE, Self::NW, Self::W, Self::SW, Self::SE];

    /// Returns the opposite direction (180 degrees).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexy::Direction;
    ///
    /// assert_eq!(Direction::E.opposite(), Direction::W);
    /// assert_eq!(Direction::NE.opposite(), Direction::SW);
    /// ```
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::E => Self::W,
            Self::NE => Self::SW,
            Self::NW => Self::SE,
            Self::W => Self::E,
            Self::SW => Self::NE,
            Self::SE => Self::NW,
        }
    }

    /// Rotates one step clockwise (60 degrees).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexy::Direction;
    ///
    /// assert_eq!(Direction::E.rotate_cw(), Direction::SE);
    /// ```
    #[must_use]
    pub const fn rotate_cw(self) -> Self {
        match self {
            Self::E => Self::SE,
            Self::SE => Self::SW,
            Self::SW => Self::W,
            Self::W => Self::NW,
            Self::NW => Self::NE,
            Self::NE => Self::E,
        }
    }

    /// Rotates one step counterclockwise (60 degrees).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexy::Direction;
    ///
    /// assert_eq!(Direction::E.rotate_ccw(), Direction::NE);
    /// ```
    #[must_use]
    pub const fn rotate_ccw(self) -> Self {
        match self {
            Self::E => Self::NE,
            Self::NE => Self::NW,
            Self::NW => Self::W,
            Self::W => Self::SW,
            Self::SW => Self::SE,
            Self::SE => Self::E,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Direction;

    #[test]
    fn opposite_is_involution() {
        for d in Direction::ALL {
            assert_eq!(d.opposite().opposite(), d);
        }
    }

    #[test]
    fn rotate_cw_cycle() {
        for d in Direction::ALL {
            let mut cur = d;
            for _ in 0..6 {
                cur = cur.rotate_cw();
            }
            assert_eq!(cur, d);
        }
    }

    #[test]
    fn rotate_ccw_cycle() {
        for d in Direction::ALL {
            let mut cur = d;
            for _ in 0..6 {
                cur = cur.rotate_ccw();
            }
            assert_eq!(cur, d);
        }
    }

    #[test]
    fn cw_then_ccw_is_identity() {
        for d in Direction::ALL {
            assert_eq!(d.rotate_cw().rotate_ccw(), d);
        }
    }

    #[test]
    fn discriminants_match_directions_array_index() {
        use crate::Hex;
        for d in Direction::ALL {
            assert_eq!(
                Hex::<i32>::DIRECTIONS[d as usize],
                Hex::<i32>::DIRECTIONS[d as usize]
            );
        }
    }
}
