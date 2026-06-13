use alloc::vec;
use alloc::vec::Vec;

use super::*;

type Backend = unsafe fn(*mut u64, usize, u64);

const LENGTHS: &[usize] = &[
    0, 1, 2, 3, 4, 5, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129,
];

const VALUES: &[u64] = &[0, u64::MAX];

fn run_backend_owned(backend: Backend, word_len: usize, value: u64) -> Vec<u64> {
    let mut out = Vec::<u64>::with_capacity(word_len);

    // SAFETY:
    // - `out` has capacity for `word_len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `word_len` u64 values.
    // - The backend contract requires it to write every slot in `0..word_len`.
    unsafe {
        backend(out.as_mut_ptr(), word_len, value);
        out.set_len(word_len);
    }

    out
}

fn run_backend_in_place(backend: Backend, word_len: usize, value: u64) -> Vec<u64> {
    let mut out = vec![0xDEAD_BEEF_DEAD_BEEFu64; word_len];
    let ptr = out.as_mut_ptr();

    // SAFETY:
    // - `ptr` is valid for writes of `word_len` u64 values.
    // - The backend contract guarantees it writes every slot (no read-before-write).
    unsafe {
        backend(ptr, word_len, value);
    }

    out
}

fn assert_backend_matches_scalar(backend: Backend) {
    for &len in LENGTHS {
        for &value in VALUES {
            let expected = run_backend_owned(scalar::words, len, value);

            let actual_owned = run_backend_owned(backend, len, value);
            let actual_in_place = run_backend_in_place(backend, len, value);

            assert_eq!(
                actual_owned, expected,
                "owned mismatch: len={len}, value={value:#x}",
            );
            assert_eq!(
                actual_in_place, expected,
                "in-place mismatch: len={len}, value={value:#x}",
            );
        }
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2"
))]
#[test]
fn sse2_matches_scalar() {
    assert_backend_matches_scalar(sse2::words);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_matches_scalar() {
    assert_backend_matches_scalar(avx2::words);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar() {
    assert_backend_matches_scalar(neon::words);
}
