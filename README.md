# hexal

Axial hex coordinates for `no_std` and embedded targets.

[![Crates.io](https://img.shields.io/crates/v/hexal.svg)](https://crates.io/crates/hexal)
[![Docs](https://docs.rs/hexal/badge.svg)](https://docs.rs/hexal)
[![CI](https://github.com/crates-lurey-io/hexal/actions/workflows/test.yml/badge.svg)](https://github.com/crates-lurey-io/hexal/actions/workflows/test.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

## Overview

`hexal` provides **axial hex coordinates** `(q, r)` with no runtime dependencies
and no allocator. Every operation is pure integer arithmetic; no floats, no
lookup tables, no heap.

```
       +r
      /
-q --●-- +q
      \
       -r     (pointy-top orientation)
```

The `s` axis (`-q - r`) satisfies the cube constraint `q + r + s = 0` and is
derived on demand.

## Quick start

```rust
use hexal::{hex, Hex, Direction};

// Construct
let a = hex!(1, -1);
let b = Hex::ORIGIN;

// Distance (pure integer, no float)
assert_eq!(a.distance(b), 2);

// Neighbors
assert_eq!(a.neighbor(Direction::W), hex!(0, -1));

// Ring iterator — no allocation
let ring: Vec<_> = b.ring(2).collect();
assert_eq!(ring.len(), 12);

// Line iterator — no allocation
let line: Vec<_> = hex!(0, 0).line_to(hex!(3, 0)).collect();
assert_eq!(line.len(), 4);
```

## Offset coordinates

For screen/storage mapping, convert via one of four standard schemes:

| Type     | Orientation | Shift |
|----------|-------------|-------|
| `OddR`   | Pointy-top  | Odd rows shifted right |
| `EvenR`  | Pointy-top  | Even rows shifted right |
| `OddQ`   | Flat-top    | Odd cols shifted down |
| `EvenQ`  | Flat-top    | Even cols shifted up |

```rust
use hexal::{hex, OddR};

let offset = hex!(1, 2).to_offset::<OddR>();
assert_eq!(offset.col, 2);
assert_eq!(offset.row, 2);
assert_eq!(offset.to_hex(), hex!(1, 2));
```

## Features

| Feature | Description |
|---------|-------------|
| `serde` | `Serialize`/`Deserialize` on `Hex` and `OffsetHex` |
| `ixy`   | `From`/`Into` with `ixy::Pos<T>` via `OddR` offset |

## Type aliases

| Alias    | Type      | Use case |
|----------|-----------|----------|
| `HexI`   | `Hex<i32>` | General purpose (default) |
| `HexI16` | `Hex<i16>` | Large embedded maps |
| `HexI8`  | `Hex<i8>`  | Tiny grids (±127 per axis) |

## `no_std`

`hexal` is unconditionally `no_std`. No `extern crate alloc` is needed; all
iterators are stack-only.

## MSRV

Rust 1.87 (edition 2024).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
