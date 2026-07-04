# Changelog

All notable changes to `hexal` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-07-04

### Changed

- Relaxed `ixy` from an exact `=0.6.1` pin to a normal caret requirement (`"0.6.1"`). The exact
  pin was a necessary workaround while `ixy` was still on its `0.6.0-alpha.N` prerelease track,
  where Cargo's caret matching against prereleases is unsafe. Now that `ixy` has a real stable
  release, normal Cargo `0.y.z` caret semantics already prevent the same hazard (caret only floats
  the patch component, never `y`), so keeping the exact pin only costs a forced republish on every
  future `ixy` patch and version-unification risk for consumers depending on both crates directly.

## [0.1.0] - 2026-07-04

Pre-1.0 stable release. No API breaking changes beyond the bug fix below (the fix changes
output for negative offset coordinates, which were wrong before).

### Fixed

- **`OffsetHex` conversions were wrong for negative rows/columns.** The internal `parity()`
  helper computed `v - (v / two) * two` to get `v`'s low bit, but Rust's `/` truncates toward
  zero, so for negative odd `v` (e.g. `-1`) this returned `-1` instead of the canonical `1`,
  silently flipping the shift direction for `OddR`/`EvenR`/`OddQ`/`EvenQ` offset coordinates with
  negative axial input. Existing tests only round-tripped through the same buggy `parity()` in
  both directions, so they passed despite the bug; added a regression test that checks against
  the canonical Red Blob Games formula independently (via `rem_euclid`) for negative rows. Fixed
  by computing `(v % two).abs()` instead, which is correct for any sign.

### Changed

- `ixy` dependency requirement tightened from a loose `"0.6.0-alpha.7"` (caret) to an exact
  `"=0.6.1"` pin. Because both crates live on pre-1.0 version tracks, the loose requirement had
  already silently resolved upward to `0.6.0-alpha.8` in this crate's own `Cargo.lock` - the same
  drift hazard `grixy` hit and fixed with exact-pinning in its own alpha.8 release.
- `missing_docs`/`unreachable_pub`/`unused_qualifications` lints promoted from `warn` to `deny`,
  matching the rest of the ecosystem (ixy/gem/grixy/framepace).
- Added `#![cfg_attr(docsrs, feature(doc_cfg))]` for consistency with sibling crates' docs.rs
  configuration.

## [0.1.0-alpha.2] - 2026-06-28

### Fixed

- Docs generation on crates.io

## [0.1.0-alpha.1] - 2026-06-28

### Added

- `Hex<T>` — axial hex coordinate with `repr(C)` layout and `i32` default.
- `Direction` — six-variant enum (pointy-top, E=0 counterclockwise to SE=5).
- `HexRing<T>` — stack-allocated ring iterator; `ring(r)` yields `6*r` hexes.
- `HexRange<T>` — stack-allocated filled-range iterator; `range(r)` yields `3r²+3r+1` hexes.
- `HexLine<T>` — stack-allocated line iterator using integer-only interpolation.
- `OffsetHex<T, S>` — offset coordinate storage with `OddR`, `EvenR`, `OddQ`, `EvenQ` schemes.
- `hex!(q, r)` macro for `const`-compatible construction.
- Type aliases `HexI` (`i32`), `HexI16`, `HexI8`.
- Optional `serde` feature: `Serialize`/`Deserialize` on `Hex` and `OffsetHex`.
- Optional `ixy` feature: `From`/`Into` with `ixy::Pos<T>` via `OddR` offset.
- Property tests via `proptest` covering distance symmetry, ring/range count formulas,
  line count invariant, rotation identity, and reflect involution.
