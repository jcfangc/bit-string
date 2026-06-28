use alloc::vec::Vec;

use super::*;

type Backend = unsafe fn(*mut u64, *const u64, usize, usize);

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
    &[
        0, 1, 2, 3, 4, 5, 6, 7, //
        8, 9, 10, 11, 12, 13, 14, 15,
    ],
];

const AMOUNTS: &[usize] = &[0, 1, 2, 7, 8, 31, 32, 63, 64, 65, 127, 128, 129, 191, 256];

fn run_backend_owned(backend: Backend, src: &[u64], amount: usize) -> Vec<u64> {
    let len = src.len();
    let mut out = Vec::<u64>::with_capacity(len);

    // SAFETY:
    // - `out` has capacity for `len` u64 values.
    // - `src` is valid for reads of `len` u64 values.
    // - `out.as_mut_ptr()` is valid for writes of `len` u64 values.
    // - `out` is freshly allocated, so it cannot overlap `src`.
    // - The backend contract requires it to write every slot in `0..len`.
    unsafe {
        backend(out.as_mut_ptr(), src.as_ptr(), len, amount);
        out.set_len(len);
    }

    out
}

fn run_backend_in_place(backend: Backend, src: &[u64], amount: usize) -> Vec<u64> {
    let len = src.len();
    let mut out = src.to_vec();
    let out_ptr = out.as_mut_ptr();

    // SAFETY:
    // - `out_ptr` is valid for reads and writes of `len` u64 values.
    // - `dst == src` is allowed by the backend contract.
    unsafe {
        backend(out_ptr, out_ptr.cast_const(), len, amount);
    }

    out
}

fn assert_backend_matches_scalar(backend: Backend) {
    for &src in CASES {
        for &amount in AMOUNTS {
            let expected = run_backend_owned(scalar::words, src, amount);

            let actual_owned = run_backend_owned(backend, src, amount);
            let actual_in_place = run_backend_in_place(backend, src, amount);

            assert_eq!(
                actual_owned, expected,
                "owned mismatch: src = {src:?}, amount = {amount}",
            );
            assert_eq!(
                actual_in_place, expected,
                "in-place mismatch: src = {src:?}, amount = {amount}",
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
