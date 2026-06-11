use alloc::vec::Vec;

use super::*;

type Backend = unsafe fn(*mut u64, *const u64, *const u64, usize);

const CASES: &[&[u64]] = &[
    &[],
    &[0],
    &[u64::MAX],
    &[0, u64::MAX],
    &[0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA],
    &[0, 1, u64::MAX, 0x0123_4567_89AB_CDEF, 0xFEDC_BA98_7654_3210],
    &[
        0xFFFF_0000_FFFF_0000,
        0x0000_FFFF_0000_FFFF,
        0x3333_3333_3333_3333,
        0xCCCC_CCCC_CCCC_CCCC,
        0x1234_5678_9ABC_DEF0,
        0x0FED_CBA9_8765_4321,
        u64::MAX,
    ],
    &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
];

fn rhs_for(len: usize) -> Vec<u64> {
    (0..len)
        .map(|i| {
            let x = i as u64;

            x.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .rotate_left((i % 63) as u32)
                ^ 0xA5A5_A5A5_A5A5_A5A5
        })
        .collect()
}

fn run_backend_owned(backend: Backend, lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
    debug_assert_eq!(lhs.len(), rhs.len());

    let len = lhs.len();
    let mut out = Vec::<u64>::with_capacity(len);

    // SAFETY:
    // - `out` has capacity for `len` u64 values.
    // - `lhs` and `rhs` are valid for reads of `len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `len` u64 values.
    // - `out` is freshly allocated, so it does not overlap `lhs` or `rhs`.
    // - The backend contract requires it to write every slot in `0..len`.
    unsafe {
        backend(out.as_mut_ptr(), lhs.as_ptr(), rhs.as_ptr(), len);
        out.set_len(len);
    }

    out
}

fn run_backend_in_place(backend: Backend, lhs: &[u64], rhs: &[u64]) -> Vec<u64> {
    debug_assert_eq!(lhs.len(), rhs.len());

    let len = lhs.len();
    let mut out = lhs.to_vec();
    let out_ptr = out.as_mut_ptr();

    // SAFETY:
    // - `out_ptr` is valid for reads and writes of `len` u64 values.
    // - `rhs` is valid for reads of `len` u64 values.
    // - `dst == lhs` is allowed by the backend contract.
    // - `rhs` does not overlap `out`.
    unsafe {
        backend(out_ptr, out_ptr.cast_const(), rhs.as_ptr(), len);
    }

    out
}

fn assert_backend_matches_scalar(backend: Backend) {
    for &lhs in CASES {
        let rhs = rhs_for(lhs.len());

        let expected = run_backend_owned(scalar::and_words, lhs, &rhs);

        let actual_owned = run_backend_owned(backend, lhs, &rhs);
        let actual_in_place = run_backend_in_place(backend, lhs, &rhs);

        assert_eq!(
            actual_owned, expected,
            "owned mismatch: lhs = {lhs:?}, rhs = {rhs:?}",
        );

        assert_eq!(
            actual_in_place, expected,
            "in-place mismatch: lhs = {lhs:?}, rhs = {rhs:?}",
        );
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2"
))]
#[test]
fn sse2_matches_scalar() {
    assert_backend_matches_scalar(sse2::and_words);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_matches_scalar() {
    assert_backend_matches_scalar(avx2::and_words);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar() {
    assert_backend_matches_scalar(neon::and_words);
}
