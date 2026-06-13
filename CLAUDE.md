# bit-string

A `no_std` Rust crate providing a compact owned bit string type with editing, matching, and bitwise operations.

## Build & Test

```bash
cargo test              # Run all tests
cargo bench             # Run all benchmarks
cargo clippy            # Lint
```

Tests include per-function unit tests and backend-equivalence tests that verify SIMD implementations (SSE2, AVX2, NEON) match the scalar reference across a matrix of inputs and shift amounts.

## Architecture

```
src/
  lib.rs                         # #![no_std], re-exports BitString and errors
  bit_string.rs                  # BitString struct definition (Box<[u64]> + len)
    errors.rs                    # ParseBitStringError, BitStringLenMismatch
    funcs_for_share.rs           # Shared low-level helpers: bit_at, set_bit, read/write_chunk,
                                 #   mask_unused_bits, last_word_mask, word_len, etc.
    impls_for_construction.rs    # new, try_from, from_bool_iter, from_words, zeros, repeat
    impls_for_access.rs          # get, len, is_empty, to_string
    impls_for_editing.rs         # push, pop, insert, remove, set, extend, truncate, slice, etc.
    impls_for_matching.rs        # find_bits, rfind_bits, starts_with, ends_with, contains_bits, etc.
    impls_for_iter.rs            # Iterator impl (yields bools)
    impls_for_fmt.rs             # Display, Debug (outputs "1010…" string)
    impls_for_bit_ops/
      impls_for_not.rs           # not, not_assign, not_into
      impls_for_count_ones.rs    # count_ones, count_zeros
      impls_for_shl.rs           # shl, shl_assign, shl_into (left shift with zero fill)
      funcs_for_binary_core.rs   # Generic binary-op dispatch: AND, OR, XOR
        impls_for_and.rs         # and, and_assign, and_into
        impls_for_or.rs          # or, or_assign, or_into
        impls_for_xor.rs         # xor, xor_assign, xor_into
      impls_for_shr.rs           # shr, shr_assign, shr_into (right shift with zero fill)
```

### SIMD Backend Dispatch Pattern

Each bit operation follows the same multi-backend architecture:

1. **Public API** methods (`shl`, `shl_assign`, `shl_into`) delegate to a `funcs_*_core` module.
2. The **core module** provides `owned()` (allocate new) and `assign()` (in-place) entry points.
3. A **`dispatch`** function uses `#[cfg]` gates to select the best available SIMD backend at compile time:
   - `avx2` (x86/x86_64, 256-bit, 4×u64 lanes)
   - `sse2` (x86/x86_64, 128-bit, 2×u64 lanes) — gated with `not(target_feature = "avx2")`
   - `neon` (aarch64, 128-bit, 2×u64 lanes)
   - `scalar` — fallback, supports all targets
4. Each backend lives in its own `mod` annotated with `#[target_feature(enable = "...")]`.
5. **Backend equivalence tests** run the scalar backend as oracle and assert every available SIMD backend produces identical output for a matrix of inputs and amounts.

### Core Data Structure

`BitString` stores bits in a `Box<[u64]>` array. The logical bit length (`len: usize`) is tracked separately. The last `u64` word is masked via `last_word_mask` to zero bits beyond `len`. Internal word size: `WORD_BITS = u64::BITS = 64`.

### Key Conventions

- `impls_for_*` implements methods for a unified theme (`impl BitString { … }` blocks).
- `funcs_for_*` provides pure functions that serve a unified theme.
- `funcs_for_*_core` is the low-level dispatch layer for a SIMD-backed operation.
- Safety comments for `unsafe` blocks document preconditions and how they're satisfied.
- `unsafe` blocks are kept minimal — each has a comment referencing the relevant guarantees.
- Functions that work on raw pointers are `unsafe` and document their safety contract in doc comments.
- Tests are structured as `mod tests_for_*` submodules inside each feature module.

## Key Dependencies

- `int-interval` — interval type for editing operations (slice, remove, replace)
- `codspeed-divan-compat` — benchmarking framework
- `bitvec_simd` — dev-dependency for benchmark comparisons
