
# bit-string

[![Coverage](https://codecov.io/gh/jcfangc/bit-string/branch/main/graph/badge.svg)](https://codecov.io/gh/jcfangc/bit-string)

`bit-string` provides an owned bit string type for Rust.

A bit string is a sequence type whose elements are bits. It is intended to feel close to primitive bitwise values where possible, while also behaving like a growable sequence.

The core type is `BitString`.

## Features

- Owned bit string storage.
- Bitwise operations:
  - `and_bits`
  - `or_bits`
  - `xor_bits`
  - `not_bits`
- Bit counting:
  - `count_ones`
  - `count_zeros`
- Editing operations:
  - `push`
  - `pop`
  - `insert`
  - `remove`
  - `truncate`
  - `clear`
  - `push_bits`
  - `insert_bits`
  - `replace_interval`
  - `drain_interval`
  - `slice`
- Matching operations:
  - `starts_with`
  - `ends_with`
  - `contains_bits`
  - `find_bits`
  - `rfind_bits`
  - `strip_prefix`
  - `strip_suffix`

## Example

```rust
use bit_string::BitString;

let lhs = BitString::try_from("1010").unwrap();
let rhs = BitString::try_from("1100").unwrap();

let and = lhs.and_bits(&rhs).unwrap();
let or = lhs.or_bits(&rhs).unwrap();
let not = lhs.not_bits();

assert_eq!(and.to_string(), "1000");
assert_eq!(or.to_string(), "1110");
assert_eq!(not.to_string(), "0101");
```

## Design goal

The goal of this crate is simple: make bit strings usable as ordinary sequence values.

In the short term, `BitString` focuses on compact storage, bitwise operations, editing, slicing, and matching.

In the long term, this crate will try to fill out the operations people expect from `String` and other sequence-like types, while keeping bit-level operations explicit and efficient.

## Status

This crate is still early. APIs may change before the first stable release.

## License

Licensed under either of:

* MIT license
* Apache License, Version 2.0

at your option.