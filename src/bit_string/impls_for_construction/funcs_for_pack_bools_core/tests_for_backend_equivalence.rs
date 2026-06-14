use alloc::vec::Vec;

use super::*;

type Backend = unsafe fn(*mut u64, *const u8, usize);

const LENGTHS: &[usize] = &[
    0, 1, 2, 3, 4, 5, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256, 257,
];

/// Generate a `len`-byte slice of pseudo-random 0/1 values seeded by `len`.
fn src_bytes(len: usize) -> Vec<u8> {
    let mut state = len.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(0x9E37);
            // Use overflow to produce a bit that varies with the seed.
            (state >> 32) as u8 & 1
        })
        .collect()
}

/// Run a backend — allocate a fresh vec, call `words`, return the packed words.
fn run_backend(backend: Backend, src: &[u8]) -> Vec<u64> {
    let bit_len = src.len();
    let word_len = if bit_len == 0 {
        0
    } else {
        (bit_len - 1) / 64 + 1
    };
    let mut out = Vec::<u64>::with_capacity(word_len);

    // SAFETY:
    // - `out` has capacity for `word_len` u64 values.
    // - `src.as_ptr()` is valid for reads of `bit_len` u8 values.
    // - The backend contract requires it to write every slot.
    unsafe {
        backend(out.as_mut_ptr(), src.as_ptr(), bit_len);
        out.set_len(word_len);
    }

    out
}

fn assert_backend_matches_scalar(backend: Backend) {
    for &len in LENGTHS {
        let src = src_bytes(len);
        let expected = run_backend(scalar::words, &src);

        let actual = run_backend(backend, &src);

        assert_eq!(actual, expected, "mismatch: len={len}, src={src:?}",);

        // Also verify that the backend output round-trips through
        // `owned` (which applies mask_unused).
        let owned = super::bools_core(src.as_ptr(), len);
        assert_eq!(&*owned, &*expected, "owned mismatch vs expected: len={len}",);
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
