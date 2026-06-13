use alloc::vec::Vec;

use super::*;

type Backend = unsafe fn(*mut u64, *const u8, usize) -> Option<(usize, u8)>;

const LENGTHS: &[usize] = &[
    0, 1, 2, 3, 4, 5, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256, 257,
];

/// Generate a `len`-byte string of pseudo-random '0'/'1' chars seeded by `len`.
fn src_bytes(len: usize) -> Vec<u8> {
    let mut state = len.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(0x9E37);
            if ((state >> 32) as u8) & 1 == 0 {
                b'0'
            } else {
                b'1'
            }
        })
        .collect()
}

/// Run a backend — allocate a fresh vec, pack, return the words.
fn run_backend(backend: Backend, src: &[u8]) -> Result<Vec<u64>, (usize, u8)> {
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
    unsafe {
        let err = backend(out.as_mut_ptr(), src.as_ptr(), bit_len);
        if let Some((idx, byte)) = err {
            return Err((idx, byte));
        }
        out.set_len(word_len);
    }

    Ok(out)
}

fn assert_backend_matches_scalar(backend: Backend) {
    for &len in LENGTHS {
        let src = src_bytes(len);
        let expected = run_backend(scalar::words, &src).unwrap();

        let actual = run_backend(backend, &src).unwrap();

        assert_eq!(actual, expected, "mismatch: len={len}",);

        // Also verify `owned()` round-trip (applies mask_unused).
        let owned = super::owned(src.as_ptr(), len).unwrap();
        assert_eq!(&*owned, &*expected, "owned mismatch: len={len}",);
    }
}

fn assert_backend_reports_error(backend: Backend) {
    // Insert an invalid byte at various positions.
    for &len in &[1usize, 8, 16, 32, 64, 65, 128] {
        let mut src = src_bytes(len);
        if len == 0 {
            continue;
        }
        // Put an 'x' in the middle.
        let pos = len / 2;
        src[pos] = b'x';

        let result = run_backend(backend, &src);
        assert!(result.is_err(), "should error: len={len}, pos={pos}");
        let (idx, byte) = result.unwrap_err();
        assert_eq!(byte, b'x', "wrong error byte: len={len}");
        assert!(idx <= pos, "error index {idx} > injection pos {pos}");
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
    target_feature = "sse2"
))]
#[test]
fn sse2_reports_error() {
    assert_backend_reports_error(sse2::words);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_matches_scalar() {
    assert_backend_matches_scalar(avx2::words);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_reports_error() {
    assert_backend_reports_error(avx2::words);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar() {
    assert_backend_matches_scalar(neon::words);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_reports_error() {
    assert_backend_reports_error(neon::words);
}
