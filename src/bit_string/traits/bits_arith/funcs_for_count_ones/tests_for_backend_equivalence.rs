use super::*;

type Backend = unsafe fn(*const u64, usize) -> usize;

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

fn run_backend(backend: Backend, src: &[u64]) -> usize {
    // SAFETY:
    // - `src.as_ptr()` is valid for reads of `src.len()` u64 values.
    unsafe { backend(src.as_ptr(), src.len()) }
}

fn assert_backend_matches_scalar(backend: Backend) {
    for &src in CASES {
        let expected = run_backend(scalar::count_words, src);
        let actual = run_backend(backend, src);

        assert_eq!(actual, expected, "src = {src:?}");
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_matches_scalar() {
    assert_backend_matches_scalar(avx2::count_words);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
#[test]
fn ssse3_matches_scalar() {
    assert_backend_matches_scalar(ssse3::count_words);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar() {
    assert_backend_matches_scalar(neon::count_words);
}
