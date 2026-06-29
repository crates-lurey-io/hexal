# Agent Guidelines for hexal

## Rust API Guidelines

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html).

Key rules enforced by CI:

- `unsafe_code` is `forbid`; no unsafe blocks anywhere.
- `#[must_use]` on all methods returning a meaningful value.
- Dual-licensed `MIT OR Apache-2.0`.
- Keep `CHANGELOG.md` up to date with every public API change.
- All public items must have doc comments with at least one `# Examples` block.

## Code style

- `just check` before committing (format + clippy with all pedantic and nursery
  lints as errors). Requires `just` and `cargo-nextest` installed locally.
- `just test-all` to run all tests including doctests.
- No `eprintln!` or `println!` in library code.

## Design invariants

- `Hex<T>` uses axial coordinates `(q, r)`. The `s` axis (`-q - r`) is derived
  on demand and never stored.
- All iterators (`HexRing`, `HexRange`, `HexLine`) are stack-allocated and
  implement `FusedIterator`. No heap anywhere.
- `HexLine<T>` intermediate computations scale by `2 * distance`, so overflow
  is possible for small integer types (`i8`, `i16`) over long lines. The
  recommended type for general use is `i32`.
- The `OffsetScheme` and `int::SignedInt` traits are sealed; they cannot be
  implemented by downstream crates.
- The `ixy` feature adds `From`/`Into` conversions via `OddR` offset.

## No interactive jj/git

Never use `-i`/`--interactive` flags. Always pass `-m` to `jj describe`/`jj new`.
