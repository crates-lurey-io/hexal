//! Offset coordinate schemes for screen/storage mapping.
//!
//! Offset coordinates are more natural for square-grid storage (e.g., a 2D
//! array) but require conversion to/from axial for any hex arithmetic.
//!
//! Four standard schemes are provided:
//!
//! | Type | Orientation | Shift |
//! |------|-------------|-------|
//! | [`OddR`]  | Pointy-top | Odd rows shifted right |
//! | [`EvenR`] | Pointy-top | Even rows shifted right |
//! | [`OddQ`]  | Flat-top   | Odd cols shifted down |
//! | [`EvenQ`] | Flat-top   | Odd cols shifted up |

use core::marker::PhantomData;

use crate::{Hex, int::SignedInt, internal::Sealed};

// ── Scheme markers ────────────────────────────────────────────────────────────

/// Pointy-top orientation, odd rows shifted right.
pub struct OddR;
/// Pointy-top orientation, even rows shifted right.
pub struct EvenR;
/// Flat-top orientation, odd columns shifted down.
pub struct OddQ;
/// Flat-top orientation, even columns shifted up.
pub struct EvenQ;

impl Sealed for OddR {}
impl Sealed for EvenR {}
impl Sealed for OddQ {}
impl Sealed for EvenQ {}

// ── OffsetScheme trait ────────────────────────────────────────────────────────

/// Conversion scheme between axial and offset coordinates.
///
/// This trait is sealed; it cannot be implemented outside of `hexal`.
#[allow(private_bounds)]
pub trait OffsetScheme<T: SignedInt>: Sealed + Sized {
    /// Converts an axial [`Hex`] to offset coordinates under this scheme.
    fn from_axial(hex: Hex<T>) -> OffsetHex<T, Self>;
    /// Converts offset coordinates back to an axial [`Hex`].
    fn to_axial(offset: OffsetHex<T, Self>) -> Hex<T>;
}

// ── OffsetHex ─────────────────────────────────────────────────────────────────

/// A hex coordinate stored as `(col, row)` under a specific offset scheme.
///
/// The scheme `S` is a zero-sized marker type ([`OddR`], [`EvenR`], [`OddQ`],
/// [`EvenQ`]) that encodes the offset convention.
///
/// ## Examples
///
/// ```rust
/// use hexal::{hex, OddR};
///
/// let offset = hex!(1, 2).to_offset::<OddR>();
/// assert_eq!(offset.col, 2);
/// assert_eq!(offset.row, 2);
/// assert_eq!(offset.to_hex(), hex!(1, 2));
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffsetHex<T: SignedInt, S: OffsetScheme<T>> {
    /// Column in the offset grid.
    pub col: T,
    /// Row in the offset grid.
    pub row: T,
    _scheme: PhantomData<S>,
}

impl<T: SignedInt, S: OffsetScheme<T>> OffsetHex<T, S> {
    /// Creates a new offset hex at `(col, row)`.
    #[must_use]
    pub const fn new(col: T, row: T) -> Self {
        Self {
            col,
            row,
            _scheme: PhantomData,
        }
    }

    /// Converts back to axial coordinates.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hexal::{hex, OddR};
    ///
    /// let h = hex!(3, -2);
    /// assert_eq!(h.to_offset::<OddR>().to_hex(), h);
    /// ```
    #[must_use]
    pub fn to_hex(self) -> Hex<T> {
        S::to_axial(self)
    }
}

// ── Conversion helpers ────────────────────────────────────────────────────────

// For pointy-top (R-offset) schemes, the shift formula is:
//   col = q + (r - shift * (r & 1)) / 2
//   row = r
// where shift = +1 for OddR (odd rows shifted right), -1 for EvenR.
//
// The inverse:
//   q = col - (row - shift * (row & 1)) / 2
//   r = row

fn parity<T: SignedInt>(v: T) -> T {
    // Returns 0 for even, 1 for odd (positive), as T.
    // We compute v & 1 via: v - (v / TWO) * TWO
    let two = T::ONE + T::ONE;
    v - (v / two) * two
}

fn r_offset_to_axial<T: SignedInt>(col: T, row: T, shift: T) -> Hex<T> {
    let two = T::ONE + T::ONE;
    let q = col - (row - shift * parity(row)) / two;
    Hex::new(q, row)
}

fn axial_to_r_offset<T: SignedInt>(hex: Hex<T>, shift: T) -> (T, T) {
    let two = T::ONE + T::ONE;
    let col = hex.q + (hex.r - shift * parity(hex.r)) / two;
    (col, hex.r)
}

// For flat-top (Q-offset) schemes:
//   col = q
//   row = r + (q - shift * (q & 1)) / 2
// Inverse:
//   q = col
//   r = row - (col - shift * (col & 1)) / 2

fn q_offset_to_axial<T: SignedInt>(col: T, row: T, shift: T) -> Hex<T> {
    let two = T::ONE + T::ONE;
    let r = row - (col - shift * parity(col)) / two;
    Hex::new(col, r)
}

fn axial_to_q_offset<T: SignedInt>(hex: Hex<T>, shift: T) -> (T, T) {
    let two = T::ONE + T::ONE;
    let row = hex.r + (hex.q - shift * parity(hex.q)) / two;
    (hex.q, row)
}

// ── Scheme impls ──────────────────────────────────────────────────────────────

impl<T: SignedInt> OffsetScheme<T> for OddR {
    fn from_axial(hex: Hex<T>) -> OffsetHex<T, Self> {
        let (col, row) = axial_to_r_offset(hex, T::ONE);
        OffsetHex::new(col, row)
    }
    fn to_axial(o: OffsetHex<T, Self>) -> Hex<T> {
        r_offset_to_axial(o.col, o.row, T::ONE)
    }
}

impl<T: SignedInt> OffsetScheme<T> for EvenR {
    fn from_axial(hex: Hex<T>) -> OffsetHex<T, Self> {
        let (col, row) = axial_to_r_offset(hex, T::NEG_ONE);
        OffsetHex::new(col, row)
    }
    fn to_axial(o: OffsetHex<T, Self>) -> Hex<T> {
        r_offset_to_axial(o.col, o.row, T::NEG_ONE)
    }
}

impl<T: SignedInt> OffsetScheme<T> for OddQ {
    fn from_axial(hex: Hex<T>) -> OffsetHex<T, Self> {
        let (col, row) = axial_to_q_offset(hex, T::ONE);
        OffsetHex::new(col, row)
    }
    fn to_axial(o: OffsetHex<T, Self>) -> Hex<T> {
        q_offset_to_axial(o.col, o.row, T::ONE)
    }
}

impl<T: SignedInt> OffsetScheme<T> for EvenQ {
    fn from_axial(hex: Hex<T>) -> OffsetHex<T, Self> {
        let (col, row) = axial_to_q_offset(hex, T::NEG_ONE);
        OffsetHex::new(col, row)
    }
    fn to_axial(o: OffsetHex<T, Self>) -> Hex<T> {
        q_offset_to_axial(o.col, o.row, T::NEG_ONE)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    fn roundtrip<S: OffsetScheme<i32>>(q: i32, r: i32) {
        let h = hex!(q, r);
        let o = h.to_offset::<S>();
        assert_eq!(o.to_hex(), h, "round-trip failed for hex!({q}, {r})");
    }

    #[test]
    fn odd_r_roundtrip() {
        for q in -3..=3 {
            for r in -3..=3 {
                roundtrip::<OddR>(q, r);
            }
        }
    }

    #[test]
    fn even_r_roundtrip() {
        for q in -3..=3 {
            for r in -3..=3 {
                roundtrip::<EvenR>(q, r);
            }
        }
    }

    #[test]
    fn odd_q_roundtrip() {
        for q in -3..=3 {
            for r in -3..=3 {
                roundtrip::<OddQ>(q, r);
            }
        }
    }

    #[test]
    fn even_q_roundtrip() {
        for q in -3..=3 {
            for r in -3..=3 {
                roundtrip::<EvenQ>(q, r);
            }
        }
    }

    #[test]
    fn odd_r_known_values() {
        // hex!(1, 2).to_offset::<OddR>() -> col=2, row=2
        let o = hex!(1, 2).to_offset::<OddR>();
        assert_eq!(o.col, 2);
        assert_eq!(o.row, 2);
    }

    #[test]
    fn negative_coords_roundtrip() {
        for q in -5..=0 {
            for r in -5..=0 {
                roundtrip::<OddR>(q, r);
                roundtrip::<EvenR>(q, r);
                roundtrip::<OddQ>(q, r);
                roundtrip::<EvenQ>(q, r);
            }
        }
    }

    #[test]
    fn zero_row_and_col() {
        roundtrip::<OddR>(0, 0);
        roundtrip::<EvenR>(0, 0);
        roundtrip::<OddQ>(0, 0);
        roundtrip::<EvenQ>(0, 0);
        roundtrip::<OddR>(3, 0);
        roundtrip::<OddR>(0, 3);
    }
}
