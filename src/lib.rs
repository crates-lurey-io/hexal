//! Axial hex coordinates for `no_std` and embedded targets.
//!
//! ## Coordinate system
//!
//! `hexal` uses **axial coordinates** `(q, r)` as the primary representation.
//! Axial is the cleanest system for arithmetic: neighbors, distance, rings, and
//! lines are all simple expressions with no lookup tables.
//!
//! The third cube axis `s` satisfies `q + r + s = 0` and is derived on demand.
//!
//! ```text
//!        +r
//!       /
//! -q --●-- +q
//!       \
//!        -r     (pointy-top orientation)
//! ```
//!
//! ## Offset coordinates
//!
//! For screen/storage mapping, use [`OffsetHex`] with one of the four standard
//! schemes: [`OddR`], [`EvenR`], [`OddQ`], [`EvenQ`]. Convert with
//! [`Hex::to_offset`] and [`OffsetHex::to_hex`].
//!
//! ## Type aliases
//!
//! [`HexI`] (`i32`), [`HexI16`] (`i16`), and [`HexI8`] (`i8`) cover the most
//! common embedded and desktop use cases.
//!
//! ## Features
//!
//! - `serde` — derive `Serialize`/`Deserialize` on [`Hex`] and [`OffsetHex`].
//! - `ixy` — enable `From`/`Into` conversions with `ixy::Pos<T>` via
//!   [`OddR`] offset (the natural screen mapping).
//!
//! ## Examples
//!
//! ```rust
//! use hexal::{hex, Hex, Direction};
//!
//! let a = hex!(1, 0);
//! let b = hex!(0, 1);
//! assert_eq!(a.distance(b), 1);
//! assert_eq!(a.neighbor(Direction::W), hex!(0, 0));
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]
#![forbid(unsafe_code)]

// The display test in hex.rs uses alloc in tests only.
#[cfg(test)]
extern crate alloc;

pub mod int;
pub(crate) mod internal;

mod dir;
pub use dir::Direction;

mod hex;
pub use hex::Hex;

mod iter;
pub use iter::{HexLine, HexRange, HexRing};

mod offset;
pub use offset::{EvenQ, EvenR, OddQ, OddR, OffsetHex, OffsetScheme};

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Hex with `i32` coordinates — the default for most use cases.
pub type HexI = Hex<i32>;

/// Hex with `i16` coordinates — suitable for large maps on embedded.
pub type HexI16 = Hex<i16>;

/// Hex with `i8` coordinates — for tiny grids (±127 per axis).
pub type HexI8 = Hex<i8>;

// ── Macros ────────────────────────────────────────────────────────────────────

/// Creates a [`Hex`] from `(q, r)` literals.
///
/// Works in `const` contexts.
///
/// # Examples
///
/// ```rust
/// use hexal::{hex, Hex};
///
/// const H: Hex = hex!(3, -1);
/// assert_eq!(H, Hex::new(3, -1));
/// ```
#[macro_export]
macro_rules! hex {
    ($q:expr, $r:expr) => {
        $crate::Hex::new($q, $r)
    };
}

// ── Property tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod proptests {
    use crate::Hex;
    use proptest::prelude::*;

    // Keep coords small enough that no intermediate value overflows i16.
    // lerp_axis computes 2*a*(dist-step) + 2*b*step + dist; with dist ≤ ~2*R
    // and coords ≤ R, the max intermediate is ~4*R^2. For i16 (max 32767),
    // R ≤ 90 keeps us safe.
    fn hex_strategy() -> impl Strategy<Value = Hex<i16>> {
        (-45_i16..=45, -45_i16..=45).prop_map(|(q, r)| hex!(q, r))
    }

    fn radius_strategy() -> impl Strategy<Value = i16> {
        0_i16..=10
    }

    proptest! {
        #[test]
        fn distance_symmetry(
            a in hex_strategy(),
            b in hex_strategy(),
        ) {
            prop_assert_eq!(a.distance(b), b.distance(a));
        }

        #[test]
        fn ring_count_formula(
            center in hex_strategy(),
            r in radius_strategy(),
        ) {
            #[allow(clippy::cast_sign_loss)]
            let expected = if r == 0 { 1 } else { 6 * r as usize };
            prop_assert_eq!(center.ring(r).count(), expected);
        }

        #[test]
        fn range_count_formula(
            center in hex_strategy(),
            r in radius_strategy(),
        ) {
            let r = i32::from(r);
            #[allow(clippy::cast_sign_loss)]
            let expected = (3 * r * r + 3 * r + 1) as usize;
            let center: Hex<i32> = Hex::new(i32::from(center.q), i32::from(center.r));
            prop_assert_eq!(center.range(r).count(), expected);
        }

        #[test]
        fn line_count_equals_distance_plus_one(
            a in hex_strategy(),
            b in hex_strategy(),
        ) {
            #[allow(clippy::cast_sign_loss)]
            let expected = a.distance(b) as usize + 1;
            prop_assert_eq!(a.line_to(b).count(), expected);
        }

        #[test]
        fn rotate_cw_six_times_identity(h in hex_strategy()) {
            let mut cur = h;
            for _ in 0..6 {
                cur = cur.rotate_cw();
            }
            prop_assert_eq!(cur, h);
        }

        #[test]
        fn reflect_q_involution(h in hex_strategy()) {
            prop_assert_eq!(h.reflect_q().reflect_q(), h);
        }
    }
}
