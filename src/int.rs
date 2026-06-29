//! Signed integer bound used throughout `hexal`.
//!
//! When the `ixy` feature is enabled, this re-exports [`ixy::int::SignedInt`]
//! directly. Otherwise a minimal local definition covers `i8` through `isize`.

// When ixy feature is enabled, re-export its trait directly.
#[cfg(feature = "ixy")]
pub use ixy::int::SignedInt;

#[cfg(not(feature = "ixy"))]
pub use self::local::SignedInt;

#[cfg(not(feature = "ixy"))]
mod local {
    use crate::internal::Sealed;
    use core::ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
    };

    /// A signed integer that can be used as a hex coordinate component.
    ///
    /// This trait is sealed; it cannot be implemented outside of `hexal`.
    #[allow(private_bounds)]
    pub trait SignedInt:
        Sealed
        + Copy
        + Eq
        + Ord
        + Add<Output = Self>
        + AddAssign
        + Sub<Output = Self>
        + SubAssign
        + Mul<Output = Self>
        + MulAssign
        + Div<Output = Self>
        + DivAssign
        + Rem<Output = Self>
        + RemAssign
        + Neg<Output = Self>
        + core::fmt::Debug
        + core::fmt::Display
    {
        /// Additive identity.
        const ZERO: Self;
        /// Multiplicative identity.
        const ONE: Self;
        /// Two — used in offset coordinate conversions.
        const TWO: Self;
        /// Negative one.
        const NEG_ONE: Self;
        /// Minimum representable value.
        const MIN: Self;
        /// Maximum representable value.
        const MAX: Self;

        /// Returns the absolute value.
        #[must_use]
        fn abs(self) -> Self;

        /// Returns the sign of the value: `-1`, `0`, or `1`.
        #[must_use]
        fn signum(self) -> Self;

        /// Clamps the value to `[lo, hi]`.
        #[must_use]
        fn clamp(self, lo: Self, hi: Self) -> Self;
    }

    macro_rules! impl_signed {
        ($($t:ty),*) => { $(
            impl Sealed for $t {}
            impl SignedInt for $t {
                const ZERO:    Self = 0;
                const ONE:     Self = 1;
                const TWO:     Self = 2;
                const NEG_ONE: Self = -1;
                const MIN:     Self = <$t>::MIN;
                const MAX:     Self = <$t>::MAX;
                fn abs(self)    -> Self { <$t>::abs(self) }
                fn signum(self) -> Self { <$t>::signum(self) }
                fn clamp(self, lo: Self, hi: Self) -> Self { Ord::clamp(self, lo, hi) }
            }
        )* };
    }
    impl_signed!(i8, i16, i32, i64, i128, isize);
}
