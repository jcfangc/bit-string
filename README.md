
# bit-string

[![Crates.io](https://img.shields.io/crates/v/bit-string.svg)](https://crates.io/crates/bit-string)
[![License](https://img.shields.io/crates/l/bit-string.svg)](https://crates.io/crates/bit-string)
[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/gh/jcfangc/bit-string)
[![Coverage](https://codecov.io/gh/jcfangc/bit-string/branch/main/graph/badge.svg)](https://codecov.io/gh/jcfangc/bit-string)

A `no_std` Rust crate providing a compact owned bit string type with construction, editing, matching, and bitwise operations.

The core type is `BitString`. Bits are packed into `Box<[u64]>` with unused high bits in the last word always zero (masked after every mutation).

## Features

- **Construction**: `new`, `zeros`, `repeat`, `from_bool_iter`, `from_words`, `try_from(&str)`
- **Bitwise ops**: `and_bits`, `or_bits`, `xor_bits`, `not_bits`, `shl`, `shr`
- **Bit counting**: `count_ones`, `count_zeros`
- **Editing**: `push`, `pop`, `insert`, `remove`, `set`, `extend`, `truncate`, `slice`, `split_off`, `replace_interval`, `retain`, `push_bit_string`, `insert_bit_string`
- **Matching**: `starts_with`, `ends_with`, `contains`, `find`, `rfind`, `strip_prefix`, `strip_suffix`
- **Access**: `get`, `len`, `is_empty`, `as_words`, `get_chunk`, `to_string`, `iter`

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

assert_eq!(a.and_bits(&b).unwrap().to_string(), "1000");
assert_eq!(a.or_bits(&b).unwrap().to_string(),  "1110");
assert_eq!((!a).to_string(),                     "0101");
assert_eq!(a.count_ones(), 2);
```

## Benchmark highlights

`count_ones` vs `bitvec_simd` on an Intel Broadwell laptop (lower is better):

| length     | bit-string | bitvec_simd |
|------------|-----------:|------------:|
| 65 bits    |    8.6 ns  |    5.7 ns   |
| 4 096 bits |   31.7 ns  |   46.4 ns   |
| 65 536 bits|  388  ns   |  629  ns    |

## Status

This crate is still early. APIs may change before the first stable release.

## License

Licensed under either of:

* MIT license
* Apache License, Version 2.0

at your option.