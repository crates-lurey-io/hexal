# Changelog

All notable changes to `hexal` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
