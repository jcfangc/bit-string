
# bit-string

[![Crates.io](https://img.shields.io/crates/v/bit-string.svg)](https://crates.io/crates/bit-string)
[![License](https://img.shields.io/crates/l/bit-string.svg)](https://crates.io/crates/bit-string)
[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/gh/jcfangc/bit-string)
[![Coverage](https://codecov.io/gh/jcfangc/bit-string/branch/main/graph/badge.svg)](https://codecov.io/gh/jcfangc/bit-string)

A `no_std` Rust crate providing a compact owned bit string type and a zero-copy view, with construction, editing, matching, comparison, hashing, and bitwise operations.

The two core types mirror the `String`/`&str` relationship:

| Type | Role | Size | `Copy` |
|------|------|------|--------|
| `BitString` | Owned bit string, backed by `Box<[u64]>` | 4×usize | No |
| `BitStr<'bs>` | Zero-copy borrowed view of a `BitString` or subrange | 3×usize (24 bytes) | **Yes** |

Bits are packed little-endian into `u64` words. Unused high bits in the last word are always zero — enforced after every mutation on `BitString` and never observable through `BitStr`.

## Features

- **Construction**: `new`, `zeros`, `repeat`, `from_bool_iter`, `from_words`, `try_from(&str)`
- **Zero-copy view**: `BitStr<'bs>` — 24-byte `Copy` type returned by `as_bit_str()`, `slice()`, `slice_from()`, `slice_until()`
- **Conversion**: `to_bit_string()` — copies a `BitStr` view into an owned `BitString`
- **Bitwise ops**: `and`, `or`, `xor`, `not`, `shl`, `shr` (each with `_assign` and `_into` variants)
- **Bit counting**: `count_ones`, `count_zeros`
- **Editing**: `push`, `pop`, `insert`, `remove`, `set`, `extend`, `truncate`, `slice`, `split_off`, `replace_interval`, `retain`, `push_bit_string`, `insert_bit_string`
- **Matching**: `starts_with`, `ends_with`, `contains`, `find`, `rfind`, `strip_prefix`, `strip_suffix`
- **Comparison & hashing**: `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash` for both `BitStr` and `BitString` — lexicographic comparison with SIMD acceleration
- **Access**: `get`, `first`, `last`, `get_chunk`, `len`/`bit_len`, `is_empty`, `words`, `iter`

### SIMD backends

Bitwise operations and construction routines dispatch to SIMD backends automatically:

| Backend | Target | Width |
|---------|--------|-------|
| AVX2    | x86 / x86_64 | 256-bit (4×u64) |
| SSSE3   | x86 / x86_64 | 128-bit (2×u64) |
| NEON    | aarch64  | 128-bit (2×u64) |
| Scalar  | all targets | fallback |

Enable `target-cpu=native` via `.cargo/config.toml` to test your local CPU's best backend.

## Example

```rust
use bit_string::BitString;

let a = BitString::try_from("1010").unwrap();
let b = BitString::try_from("1100").unwrap();

// Bitwise operations
assert_eq!(a.and(&b).unwrap().to_string(), "1000");
assert_eq!(a.or(&b).unwrap().to_string(),  "1110");
assert_eq!((!a).to_string(),              "0101");
assert_eq!(a.count_ones(), 2);

// Zero-copy views via BitStr
let view = a.as_bit_str();              // &BitString → BitStr
let sub = view.slice_from(1);           // "010"
assert!(sub.starts_with(&BitString::try_from("01").unwrap().as_bit_str()));
assert_eq!(sub.count_ones(), 1);

// Comparison
use core::cmp::Ordering;
assert_eq!(a.cmp(&b), Ordering::Less);  // "1010" < "1100"

// Convert back to owned
let owned = sub.to_bit_string();
assert_eq!(owned.to_string(), "010");
```

## Benchmarks

Continuous benchmarking results are published at:

<https://jcfangc.github.io/bit-string/compare-plotly/index.html>

## Status

This crate is still early. APIs may change before the first stable release.

## License

Licensed under either of:

* MIT license
* Apache License, Version 2.0

at your option.