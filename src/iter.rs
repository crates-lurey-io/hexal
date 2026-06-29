//! Stack-allocated iterators over hex regions.

use core::iter::FusedIterator;

use crate::{Hex, int::SignedInt};

// ── HexRing ───────────────────────────────────────────────────────────────────

/// Iterator over the ring of hexes at exactly `radius` steps from a center.
///
/// Yields `6 * radius` hexes for `radius > 0`, or 1 hex (the center) for
/// `radius == 0`. No allocation.
///
/// Created by [`Hex::ring`].
pub struct HexRing<T: SignedInt> {
    /// Current position.
    current: Hex<T>,
    /// Number of steps remaining in the current edge.
    steps: T,
    /// Which of the 6 edges we are on (0–5), or 6 when exhausted.
    direction: u8,
    /// Radius; 0 is a special single-element case.
    radius: T,
    /// Whether the single center-hex (radius=0) has been yielded.
    done: bool,
}

impl<T: SignedInt> HexRing<T> {
    pub(crate) fn new(center: Hex<T>, radius: T) -> Self {
        if radius == T::ZERO {
            return Self {
                current: center,
                steps: T::ZERO,
                direction: 0,
                radius,
                done: false,
            };
        }
        // Start at center + SW * radius, then walk the 6 edges.
        let start = center + Hex::<T>::SW * radius;
        Self {
            current: start,
            steps: T::ZERO,
            direction: 0,
            radius,
            done: false,
        }
    }
}

impl<T: SignedInt> Iterator for HexRing<T> {
    type Item = Hex<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        // radius == 0: yield the center once.
        if self.radius == T::ZERO {
            self.done = true;
            return Some(self.current);
        }
        // All 6 edges done.
        if self.direction >= 6 {
            return None;
        }

        let item = self.current;

        // Advance along the current edge. Starting from the SW corner, edge i
        // is walked by moving in direction i (redblobgames algorithm).
        self.current += Hex::<T>::DIRECTIONS[self.direction as usize];
        self.steps += T::ONE;

        if self.steps == self.radius {
            self.steps = T::ZERO;
            self.direction += 1;
        }

        Some(item)
    }
}

impl<T: SignedInt> FusedIterator for HexRing<T> {}

// ── HexRange ──────────────────────────────────────────────────────────────────

/// Iterator over all hexes within `radius` steps of a center (inclusive).
///
/// Yields `3*r² + 3*r + 1` hexes. No allocation.
///
/// Created by [`Hex::range`].
pub struct HexRange<T: SignedInt> {
    center: Hex<T>,
    radius: T,
    /// Current q offset in `[-radius, radius]`.
    dq: T,
    /// Current r offset in `[max(-R, -dq-R), min(R, -dq+R)]`.
    dr: T,
    /// Upper bound of dr for the current dq.
    dr_max: T,
    done: bool,
}

impl<T: SignedInt> HexRange<T> {
    pub(crate) fn new(center: Hex<T>, radius: T) -> Self {
        let dq = -radius;
        // dr_min = max(-radius, -dq - radius) = max(-R, -(-R) - R) = max(-R, 0) = 0
        // dr_max = min(radius, -dq + radius) = min(R, R + R) = R
        let dr_min = if -radius > -dq - radius {
            -radius
        } else {
            -dq - radius
        };
        let dr_max = if radius < -dq + radius {
            radius
        } else {
            -dq + radius
        };
        Self {
            center,
            radius,
            dq,
            dr: dr_min,
            dr_max,
            done: false,
        }
    }
}

impl<T: SignedInt> Iterator for HexRange<T> {
    type Item = Hex<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if self.dq > self.radius {
            return None;
        }

        let item = self.center + Hex::new(self.dq, self.dr);

        // Advance dr.
        if self.dr < self.dr_max {
            self.dr += T::ONE;
        } else {
            // Move to next q column.
            self.dq += T::ONE;
            if self.dq > self.radius {
                self.done = true;
                return Some(item);
            }
            // dr_min = max(-radius, -dq - radius)
            let a = -self.radius;
            let b = -self.dq - self.radius;
            self.dr = if a > b { a } else { b };
            // dr_max = min(radius, -dq + radius)
            let c = self.radius;
            let d = -self.dq + self.radius;
            self.dr_max = if c < d { c } else { d };
        }

        Some(item)
    }
}

impl<T: SignedInt> FusedIterator for HexRange<T> {}

// ── HexLine ───────────────────────────────────────────────────────────────────

/// Iterator over the hex line from one hex to another.
///
/// Yields `distance + 1` hexes including both endpoints.
/// Uses scaled integer interpolation — no float, no alloc.
///
/// Created by [`Hex::line_to`].
pub struct HexLine<T: SignedInt> {
    start: Hex<T>,
    end: Hex<T>,
    distance: T,
    /// Number of steps taken so far (0..=distance).
    step: T,
}

impl<T: SignedInt> HexLine<T> {
    pub(crate) fn new(start: Hex<T>, end: Hex<T>) -> Self {
        let distance = start.distance(end);
        Self {
            start,
            end,
            distance,
            step: T::ZERO,
        }
    }

    /// Integer linear interpolation on a single axis.
    ///
    /// Computes `round(a + (b-a)*step/dist)` using floor division so that
    /// negative values round correctly (Rust `/` truncates toward zero, which
    /// breaks for half-negative fractions).
    fn lerp_axis(a: T, b: T, step: T, dist: T) -> T {
        // Exact numerator over denominator `dist`: a*(dist-step) + b*step
        let num = a * (dist - step) + b * step;
        // round(num/dist) = floor((2*num + dist) / (2*dist))
        let two = T::ONE + T::ONE;
        let numer = two * num + dist;
        let denom = two * dist;
        // Floor division for positive denom:
        if numer >= T::ZERO {
            numer / denom
        } else {
            // Move one unit toward -∞ to get floor instead of truncation.
            (numer - denom + T::ONE) / denom
        }
    }
}

impl<T: SignedInt> Iterator for HexLine<T> {
    type Item = Hex<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step > self.distance {
            return None;
        }

        let item = if self.distance == T::ZERO {
            self.start
        } else {
            let q = Self::lerp_axis(self.start.q, self.end.q, self.step, self.distance);
            let r = Self::lerp_axis(self.start.r, self.end.r, self.step, self.distance);
            Hex::new(q, r)
        };

        self.step += T::ONE;
        Some(item)
    }
}

impl<T: SignedInt> FusedIterator for HexLine<T> {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::{Hex, hex};

    // ── Ring ──────────────────────────────────────────────────────────────────

    #[test]
    fn ring_radius_0_is_center() {
        let center = hex!(1, 2);
        let v: alloc::vec::Vec<_> = center.ring(0).collect();
        assert_eq!(v, [center]);
    }

    #[test]
    fn ring_counts() {
        let c = Hex::ORIGIN;
        assert_eq!(c.ring(0).count(), 1);
        assert_eq!(c.ring(1).count(), 6);
        assert_eq!(c.ring(2).count(), 12);
        assert_eq!(c.ring(3).count(), 18);
    }

    #[test]
    fn ring_all_at_correct_distance() {
        let center = hex!(2, -1);
        for r in 0_i32..=4 {
            for h in center.ring(r) {
                assert_eq!(
                    center.distance(h),
                    r,
                    "ring({r}) hex {h} has wrong distance"
                );
            }
        }
    }

    #[test]
    fn ring_is_fused() {
        let mut it = Hex::ORIGIN.ring(1);
        // exhaust
        for _ in 0..6 {
            it.next();
        }
        assert!(it.next().is_none());
        assert!(it.next().is_none());
    }

    // ── Range ─────────────────────────────────────────────────────────────────

    #[test]
    fn range_counts() {
        let c = Hex::ORIGIN;
        assert_eq!(c.range(0).count(), 1);
        assert_eq!(c.range(1).count(), 7);
        assert_eq!(c.range(2).count(), 19);
        assert_eq!(c.range(3).count(), 37);
    }

    #[test]
    fn range_all_within_radius() {
        let center = hex!(-1, 2);
        for r in 0_i32..=3 {
            for h in center.range(r) {
                assert!(center.distance(h) <= r, "range({r}) hex {h} is too far");
            }
        }
    }

    // ── Line ──────────────────────────────────────────────────────────────────

    #[test]
    fn line_endpoints() {
        let a = hex!(0, 0);
        let b = hex!(3, -1);
        let line: alloc::vec::Vec<_> = a.line_to(b).collect();
        assert_eq!(*line.first().unwrap(), a);
        assert_eq!(*line.last().unwrap(), b);
    }

    #[test]
    fn line_count_equals_distance_plus_one() {
        let a = hex!(0, 0);
        let b = hex!(4, -2);
        #[allow(clippy::cast_sign_loss)]
        let expected = a.distance(b) as usize + 1;
        assert_eq!(a.line_to(b).count(), expected);
    }

    #[test]
    fn line_zero_distance() {
        let h = hex!(1, 1);
        let line: alloc::vec::Vec<_> = h.line_to(h).collect();
        assert_eq!(line, [h]);
    }

    #[test]
    fn line_consecutive_steps_are_neighbors() {
        let a = hex!(0, 0);
        let b = hex!(4, -2);
        let line: alloc::vec::Vec<_> = a.line_to(b).collect();
        for w in line.windows(2) {
            assert_eq!(
                w[0].distance(w[1]),
                1,
                "consecutive line hexes {:?} and {:?} are not neighbors",
                w[0],
                w[1]
            );
        }
    }

    extern crate alloc;
}
