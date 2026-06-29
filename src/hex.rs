//! The core [`Hex`] type and its methods.

use core::{
    fmt,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::{Direction, HexLine, HexRange, HexRing, int::SignedInt, offset::OffsetScheme};

/// An axial hex coordinate `(q, r)`.
///
/// ## Layout
///
/// `repr(C)` is guaranteed, matching a C struct `{ T q; T r; }`.
///
/// ## Ordering
///
/// Ordered by `r` first (row-major), then `q`. This is the natural order for
/// row-by-row iteration of a hex grid.
///
/// ## Type parameter
///
/// `T` must implement [`SignedInt`]. Defaults to `i32`.
///
/// ## Examples
///
/// ```rust
/// use hexal::{hex, Hex};
///
/// let h = hex!(2, -1);
/// assert_eq!(h.q, 2);
/// assert_eq!(h.r, -1);
/// assert_eq!(h.s(), -1);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hex<T: SignedInt = i32> {
    /// Column axis (runs northeast-southwest).
    pub q: T,
    /// Row axis (runs north-south).
    pub r: T,
}

// ── Constants ─────────────────────────────────────────────────────────────────

impl<T: SignedInt> Hex<T> {
    /// The hex at the origin `(0, 0)`.
    pub const ORIGIN: Self = Self {
        q: T::ZERO,
        r: T::ZERO,
    };

    /// Unit vector pointing East `(+1, 0)`.
    pub const E: Self = Self {
        q: T::ONE,
        r: T::ZERO,
    };
    /// Unit vector pointing Northeast `(+1, -1)`.
    pub const NE: Self = Self {
        q: T::ONE,
        r: T::NEG_ONE,
    };
    /// Unit vector pointing Northwest `(0, -1)`.
    pub const NW: Self = Self {
        q: T::ZERO,
        r: T::NEG_ONE,
    };
    /// Unit vector pointing West `(-1, 0)`.
    pub const W: Self = Self {
        q: T::NEG_ONE,
        r: T::ZERO,
    };
    /// Unit vector pointing Southwest `(-1, +1)`.
    pub const SW: Self = Self {
        q: T::NEG_ONE,
        r: T::ONE,
    };
    /// Unit vector pointing Southeast `(0, +1)`.
    pub const SE: Self = Self {
        q: T::ZERO,
        r: T::ONE,
    };

    /// All six unit direction vectors, ordered E → NE → NW → W → SW → SE.
    ///
    /// Index matches [`Direction`] variant discriminant.
    pub const DIRECTIONS: [Self; 6] = [Self::E, Self::NE, Self::NW, Self::W, Self::SW, Self::SE];
}

// ── Core methods ──────────────────────────────────────────────────────────────

impl<T: SignedInt> Hex<T> {
    /// Creates a new hex at `(q, r)`.
    ///
    /// Prefer the [`hex!`](crate::hex!) macro for literal coordinates.
    #[must_use]
    pub const fn new(q: T, r: T) -> Self {
        Self { q, r }
    }

    /// Returns the derived `s` axis value (`-q - r`).
    ///
    /// Satisfies the cube constraint `q + r + s = 0`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::hex;
    ///
    /// assert_eq!(hex!(1, 2).s(), -3);
    /// ```
    #[must_use]
    pub fn s(self) -> T {
        -self.q - self.r
    }

    /// Returns the neighbor in the given direction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::{hex, Direction};
    ///
    /// assert_eq!(hex!(1, 0).neighbor(Direction::W), hex!(0, 0));
    /// ```
    #[must_use]
    pub fn neighbor(self, dir: Direction) -> Self {
        self + Self::DIRECTIONS[dir as usize]
    }

    /// Returns all six neighbors as an array, starting East, counterclockwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::{hex, Direction};
    ///
    /// let neighbors = hex!(0, 0).neighbors();
    /// assert_eq!(neighbors[Direction::E as usize], hex!(1, 0));
    /// ```
    #[must_use]
    pub fn neighbors(self) -> [Self; 6] {
        Self::DIRECTIONS.map(|d| self + d)
    }

    /// Returns the hex distance between `self` and `other`.
    ///
    /// Uses cube coordinates: `max(|dq|, |dr|, |ds|)`.
    /// Pure integer arithmetic — no division, no float.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::hex;
    ///
    /// assert_eq!(hex!(0, 0).distance(hex!(3, -1)), 3);
    /// assert_eq!(hex!(-2, 3).distance(hex!(1, -1)), 4);
    /// ```
    #[must_use]
    pub fn distance(self, other: Self) -> T {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = (self.s() - other.s()).abs();
        if dq >= dr && dq >= ds {
            dq
        } else if dr >= ds {
            dr
        } else {
            ds
        }
    }

    /// Returns the distance from the origin (length of this hex vector).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::hex;
    ///
    /// assert_eq!(hex!(2, -1).length(), 2);
    /// ```
    #[must_use]
    pub fn length(self) -> T {
        self.distance(Self::ORIGIN)
    }

    /// Returns an iterator over the ring of hexes at exactly `radius` steps.
    ///
    /// Yields `6 * radius` hexes for `radius > 0`, or 1 hex for `radius == 0`.
    /// No allocation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::Hex;
    ///
    /// assert_eq!(Hex::ORIGIN.ring(0).count(), 1);
    /// assert_eq!(Hex::ORIGIN.ring(1).count(), 6);
    /// assert_eq!(Hex::ORIGIN.ring(2).count(), 12);
    /// ```
    #[must_use]
    pub fn ring(self, radius: T) -> HexRing<T> {
        HexRing::new(self, radius)
    }

    /// Returns an iterator over all hexes within `radius` steps (inclusive).
    ///
    /// Yields `3*r² + 3*r + 1` hexes. No allocation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::Hex;
    ///
    /// assert_eq!(Hex::ORIGIN.range(0).count(), 1);
    /// assert_eq!(Hex::ORIGIN.range(1).count(), 7);
    /// assert_eq!(Hex::ORIGIN.range(2).count(), 19);
    /// ```
    #[must_use]
    pub fn range(self, radius: T) -> HexRange<T> {
        HexRange::new(self, radius)
    }

    /// Returns an iterator over the line from `self` to `other`.
    ///
    /// Yields `distance + 1` hexes including both endpoints.
    /// Uses integer-only interpolation — no float.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::hex;
    ///
    /// let line: Vec<_> = hex!(0, 0).line_to(hex!(2, 0)).collect();
    /// assert_eq!(line.len(), 3);
    /// assert_eq!(line[0], hex!(0, 0));
    /// assert_eq!(line[2], hex!(2, 0));
    /// ```
    #[must_use]
    pub fn line_to(self, other: Self) -> HexLine<T> {
        HexLine::new(self, other)
    }

    /// Converts to offset coordinates using the given scheme.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::{hex, OddR};
    ///
    /// let offset = hex!(1, 2).to_offset::<OddR>();
    /// assert_eq!(offset.col, 2);
    /// assert_eq!(offset.row, 2);
    /// ```
    #[must_use]
    pub fn to_offset<S: OffsetScheme<T>>(self) -> crate::offset::OffsetHex<T, S> {
        S::from_axial(self)
    }

    /// Rotates 60 degrees clockwise around the origin.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::hex;
    ///
    /// assert_eq!(hex!(1, 0).rotate_cw(), hex!(0, 1));
    /// ```
    #[must_use]
    pub fn rotate_cw(self) -> Self {
        Self {
            q: -self.r,
            r: -self.s(),
        }
    }

    /// Rotates 60 degrees counterclockwise around the origin.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::hex;
    ///
    /// // E counterclockwise is NE
    /// assert_eq!(hex!(1, 0).rotate_ccw(), hex!(1, -1));
    /// ```
    #[must_use]
    pub fn rotate_ccw(self) -> Self {
        // CCW rotation: (q,r,s) → (-s,-q,-r) in cube
        // new_q = -s = q+r, new_r = -q
        Self {
            q: -self.s(),
            r: -self.q,
        }
    }

    /// Reflects across the q axis (swaps `r` and `s`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::hex;
    ///
    /// assert_eq!(hex!(1, 2).reflect_q().reflect_q(), hex!(1, 2));
    /// ```
    #[must_use]
    pub fn reflect_q(self) -> Self {
        Self {
            q: self.q,
            r: self.s(),
        }
    }

    /// Reflects across the r axis (swaps `q` and `s`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::hex;
    ///
    /// assert_eq!(hex!(1, 2).reflect_r().reflect_r(), hex!(1, 2));
    /// ```
    #[must_use]
    pub fn reflect_r(self) -> Self {
        Self {
            q: self.s(),
            r: self.r,
        }
    }

    /// Reflects across the s axis (swaps `q` and `r`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::hex;
    ///
    /// assert_eq!(hex!(1, 2).reflect_s().reflect_s(), hex!(1, 2));
    /// ```
    #[must_use]
    pub const fn reflect_s(self) -> Self {
        Self {
            q: self.r,
            r: self.q,
        }
    }
}

// ── Operators ─────────────────────────────────────────────────────────────────

impl<T: SignedInt> Add for Hex<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl<T: SignedInt> AddAssign for Hex<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.q += rhs.q;
        self.r += rhs.r;
    }
}

impl<T: SignedInt> Sub for Hex<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}

impl<T: SignedInt> SubAssign for Hex<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.q -= rhs.q;
        self.r -= rhs.r;
    }
}

impl<T: SignedInt> Neg for Hex<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            q: -self.q,
            r: -self.r,
        }
    }
}

impl<T: SignedInt> Mul<T> for Hex<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self {
        Self {
            q: self.q * rhs,
            r: self.r * rhs,
        }
    }
}

impl<T: SignedInt> MulAssign<T> for Hex<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.q *= rhs;
        self.r *= rhs;
    }
}

// ── Ordering ──────────────────────────────────────────────────────────────────

impl<T: SignedInt> PartialOrd for Hex<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: SignedInt> Ord for Hex<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Row-major: r primary, q secondary.
        self.r.cmp(&other.r).then(self.q.cmp(&other.q))
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

impl<T: SignedInt> fmt::Display for Hex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.q, self.r)
    }
}

// ── Conversions ───────────────────────────────────────────────────────────────

impl<T: SignedInt> From<(T, T)> for Hex<T> {
    fn from((q, r): (T, T)) -> Self {
        Self::new(q, r)
    }
}

impl<T: SignedInt> From<[T; 2]> for Hex<T> {
    fn from([q, r]: [T; 2]) -> Self {
        Self::new(q, r)
    }
}

impl<T: SignedInt> From<Hex<T>> for (T, T) {
    fn from(h: Hex<T>) -> Self {
        (h.q, h.r)
    }
}

impl<T: SignedInt> From<Hex<T>> for [T; 2] {
    fn from(h: Hex<T>) -> Self {
        [h.q, h.r]
    }
}

// ── ixy interop ───────────────────────────────────────────────────────────────

#[cfg(feature = "ixy")]
impl<T> From<Hex<T>> for ixy::Pos<T>
where
    T: SignedInt + ixy::int::Int,
{
    fn from(h: Hex<T>) -> Self {
        let o = h.to_offset::<crate::offset::OddR>();
        #[allow(clippy::use_self)]
        ixy::Pos::new(o.col, o.row)
    }
}

#[cfg(feature = "ixy")]
impl<T> From<ixy::Pos<T>> for Hex<T>
where
    T: SignedInt + ixy::int::Int,
{
    fn from(p: ixy::Pos<T>) -> Self {
        crate::offset::OffsetHex::<T, crate::offset::OddR>::new(p.x, p.y).to_hex()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    #[test]
    fn s_axis_constraint() {
        let h = hex!(3, -1);
        assert_eq!(h.q + h.r + h.s(), 0);
    }

    #[test]
    fn distance_symmetry() {
        let a = hex!(2, -1);
        let b = hex!(-1, 3);
        assert_eq!(a.distance(b), b.distance(a));
    }

    #[test]
    fn distance_known_values() {
        assert_eq!(hex!(0, 0).distance(hex!(3, -1)), 3);
        assert_eq!(hex!(-2, 3).distance(hex!(1, -1)), 4);
    }

    #[test]
    fn length_matches_distance_from_origin() {
        let h = hex!(2, -1);
        assert_eq!(h.length(), h.distance(hex!(0, 0)));
    }

    #[test]
    fn neighbor_all_directions() {
        let center = hex!(0, 0);
        assert_eq!(center.neighbor(Direction::E), hex!(1, 0));
        assert_eq!(center.neighbor(Direction::NE), hex!(1, -1));
        assert_eq!(center.neighbor(Direction::NW), hex!(0, -1));
        assert_eq!(center.neighbor(Direction::W), hex!(-1, 0));
        assert_eq!(center.neighbor(Direction::SW), hex!(-1, 1));
        assert_eq!(center.neighbor(Direction::SE), hex!(0, 1));
    }

    #[test]
    fn rotate_cw_six_times_is_identity() {
        let h = hex!(3, -1);
        let mut cur = h;
        for _ in 0..6 {
            cur = cur.rotate_cw();
        }
        assert_eq!(cur, h);
    }

    #[test]
    fn rotate_ccw_six_times_is_identity() {
        let h = hex!(2, 3);
        let mut cur = h;
        for _ in 0..6 {
            cur = cur.rotate_ccw();
        }
        assert_eq!(cur, h);
    }

    #[test]
    fn reflect_involutions() {
        let h = hex!(1, 2);
        assert_eq!(h.reflect_q().reflect_q(), h);
        assert_eq!(h.reflect_r().reflect_r(), h);
        assert_eq!(h.reflect_s().reflect_s(), h);
    }

    #[test]
    fn arithmetic_ops() {
        let a = hex!(1, 2);
        let b = hex!(3, -1);
        assert_eq!(a + b, hex!(4, 1));
        assert_eq!(a - b, hex!(-2, 3));
        assert_eq!(-a, hex!(-1, -2));
        assert_eq!(a * 3, hex!(3, 6));
    }

    #[test]
    fn ordering_row_major() {
        let a = hex!(5, 0);
        let b = hex!(0, 1);
        // r=0 < r=1, so a < b
        assert!(a < b);
    }

    #[test]
    fn display() {
        assert_eq!(alloc::format!("{}", hex!(1, -2)), "(1, -2)");
    }

    #[test]
    fn conversions() {
        let h: Hex = (1, 2).into();
        assert_eq!(h, hex!(1, 2));
        let h: Hex = [3, 4].into();
        assert_eq!(h, hex!(3, 4));
        let t: (i32, i32) = hex!(5, 6).into();
        assert_eq!(t, (5, 6));
        let a: [i32; 2] = hex!(7, 8).into();
        assert_eq!(a, [7, 8]);
    }

    #[test]
    fn i8_boundaries() {
        let h = Hex::<i8>::new(i8::MAX, i8::MIN);
        assert_eq!(h.q, i8::MAX);
        assert_eq!(h.r, i8::MIN);
    }

    extern crate alloc;
}
