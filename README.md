
# bit-string

[![Crates.io](https://img.shields.io/crates/v/bit-string.svg)](https://crates.io/crates/bit-string)
[![License](https://img.shields.io/crates/l/bit-string.svg)](https://crates.io/crates/bit-string)
[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/gh/jcfangc/bit-string)
[![Coverage](https://codecov.io/gh/jcfangc/bit-string/branch/main/graph/badge.svg)](https://codecov.io/gh/jcfangc/bit-string)

A `no_std` + `alloc` Rust crate providing a compact owned bit string and a zero-copy view, with construction, editing, matching, comparison, and bitwise operations — all accelerated by runtime SIMD dispatch (AVX2, SSSE3, NEON).

## Quick start

```rust
use bit_string::BitString;

// Parse from a binary string
let bits = BitString::try_from("1010_0011").unwrap();
assert_eq!(bits.to_string(), "10100011");
assert_eq!(bits.len(), 8);
assert_eq!(bits.count_ones(), 4);

// Build programmatically
let mut b = BitString::zeros(10);           // "0000000000"
b.set(0, true);                              // "1000000000"
b.set(9, true);                              // "1000000001"
b.push(true);                                // "10000000011"

// Bitwise operations
let a = BitString::try_from("1010").unwrap();
let c = BitString::try_from("1100").unwrap();
assert_eq!(a.and(&c).unwrap().to_string(), "1000");
assert_eq!(a.or(&c).unwrap().to_string(),  "1110");
assert_eq!(a.xor(&c).unwrap().to_string(), "0110");
assert_eq!((!a).to_string(),               "0101");
```

**Core types** mirror `String` / `&str`:

| Type | Role | Size | `Copy` |
|------|------|------|--------|
| `BitString` | Owned, `Box<[u64]>` backing | 4×usize | No |
| `BitStr<'bs>` | Zero-copy borrowed view | 3×usize | **Yes** |

Bits are packed little-endian into `u64` words. Unused high bits in the last word are always zero.

## API by category

### Construction

```rust
use bit_string::BitString;

// From a binary string literal
let a = BitString::try_from("0101").unwrap();     // "0101"
let b = BitString::try_from("0101_1110").unwrap(); // "01011110" (underscores ignored)

// Pre-allocated
let z = BitString::zeros(100);    // 100 zero bits
let o = BitString::ones(64);      // 64 one bits
let r = BitString::repeat(true, 42); // 42 one bits

// From an iterator
let v: BitString = (0..8).map(|i| i % 2 == 0).collect();
// "10101010"
```

### Zero-copy views (`BitStr`)

```rust
use bit_string::BitString;
use int_interval::UsizeCO;

let bits = BitString::try_from("1100_1010").unwrap();

// Full view
let view = bits.as_bit_str();           // &BitStr = "11001010"
assert_eq!(view.bit_len(), 8);

// Sub-slice — zero-copy, O(1)
let sub = view.slice(UsizeCO::try_new(2, 6).unwrap()); // "0010"
assert!(sub.starts_with(&BitString::try_from("00").unwrap().as_bit_str()));

// Convert back to owned
let owned = sub.to_bit_string();        // BitString = "0010"
```

### Querying bits

```rust
let bits = BitString::try_from("1011").unwrap();

// Individual bits
assert_eq!(bits.get(0), Some(true));    // index 0 = leftmost
assert_eq!(bits.get(3), Some(true));
assert_eq!(bits.first(), Some(true));
assert_eq!(bits.last(), Some(true));

// Bulk counting
assert_eq!(bits.count_ones(), 3);
assert_eq!(bits.count_zeros(), 1);
assert_eq!(bits.leading_zeros(), 0);    // starts with '1'
assert_eq!(bits.trailing_ones(), 2);    // ends with "11"

// Bit-level matching
let pattern = BitString::try_from("10").unwrap();
assert!(bits.starts_with(pattern.as_bit_str()));
```

### Editing

```rust
let mut bits = BitString::try_from("1010").unwrap();

// Single-bit operations
bits.set(0, false);                  // "0010"
let popped = bits.pop();             // → Some(false), bits = "001"
bits.push(true);                     // "0011"
bits.insert(0, true);                // "10011" (insert at front)

// Bulk operations
bits.extend(&[true, false, false]);  // "10011100"
bits.truncate(4);                    // "1001"
bits.split_off(2);                   // → BitString "01", bits = "10"

// Range operations
use int_interval::UsizeCO;
let interval = UsizeCO::try_new(1, 3).unwrap();
bits.replace_interval(interval, &BitString::ones(2)); // "111"
bits.remove(UsizeCO::try_new(0, 2).unwrap());         // "1"
```

### Matching & searching

```rust
let haystack = BitString::try_from("0010_1100").unwrap();
let needle = BitString::try_from("01").unwrap();
let np = needle.as_bit_str();

// Fixed-end checks
assert!(haystack.ends_with(np));
assert!(!haystack.starts_with(np));

// Substring search
assert!(haystack.contains(np));
assert_eq!(haystack.find(np), Some(1));   // "01" starts at index 1
assert_eq!(haystack.rfind(np), Some(5));  // last "01" at index 5

// Strip
let s = BitString::try_from("00010100").unwrap();
let stripped = s.strip_prefix(&BitString::try_from("00").unwrap().as_bit_str());
assert_eq!(stripped.unwrap().to_string(), "010100");
```

### Bitwise operations

```rust
let x = BitString::try_from("1010").unwrap();
let y = BitString::try_from("0110").unwrap();

// Consuming (`_into`) — reuses allocation
let z = x.and_into(&y).unwrap();     // "0010"
let z = y.or_into(&x).unwrap();      // "1110"
let z = y.xor_into(&x).unwrap();     // "1100"

// In-place (`_assign`)
let mut w = x.clone();
w.and_assign(&y);                    // w = "0010"

// Shift
let s = BitString::try_from("1001").unwrap();
assert_eq!(s.shl(2).to_string(), "0100");  // left shift, zero-fill
assert_eq!(s.shr(1).to_string(), "0100");  // right shift, zero-fill

// Not
assert_eq!((!s).to_string(), "0110");
```

### Comparison, hashing, iteration

```rust
use core::cmp::Ordering;
use std::collections::HashSet;

let a = BitString::try_from("100").unwrap();
let b = BitString::try_from("101").unwrap();

// Lexicographic ordering (SIMD-accelerated)
assert_eq!(a.cmp(&b), Ordering::Less);    // "100" < "101"
assert!(a < b);

// Hash — usable as HashMap/HashSet keys
let mut set = HashSet::new();
set.insert(a.clone());
assert!(set.contains(&a));

// Iterate over bits
let bits: Vec<bool> = a.iter().collect();
assert_eq!(bits, vec![true, false, false]);
```

## SIMD backends

At runtime (or compile time with the `compile-time-dispatch` feature) the crate selects the best available SIMD backend:

| Backend | Target | Width |
|---------|--------|-------|
| AVX2 | x86 / x86_64 | 256-bit (4×u64) |
| SSSE3 | x86 / x86_64 | 128-bit (2×u64) |
| NEON | aarch64 | 128-bit (2×u64) |
| Scalar | all targets | fallback |

For maximum local performance, copy the example config:

```bash
cp .cargo/config.toml.example .cargo/config.toml
```

This enables `target-cpu=native` for local builds. It is gitignored — CI already sets the appropriate flags.

## Benchmarks

Continuous benchmarking results: [jcfangc.github.io/bit-string](https://jcfangc.github.io/bit-string/compare-plotly/index.html)

## License

Licensed under either of MIT or Apache-2.0 at your option.
