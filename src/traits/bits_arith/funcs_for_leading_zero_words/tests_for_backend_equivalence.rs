use super::*;

const CASES: &[&[u64]] = &[
    &[],
    &[0],
    &[u64::MAX],
    &[1],
    &[0, u64::MAX],
    &[0, 0, u64::MAX],
    &[0, 0, 0, 0, u64::MAX],
    &[0, 0, 0, 0, 0, u64::MAX],
    &[0, 0, 0, 0, 0, 0, u64::MAX],
    &[0, 0, 0, 0, 0, 0, 0, u64::MAX],
    &[0, 0, 0, 0, 0, 0, 0, 0, u64::MAX],
    &[0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA],
    &[0, 1, u64::MAX, 0x0123_4567_89AB_CDEF, 0xFEDC_BA98_7654_3210],
    &[
        0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0,
    ],
    // All zeros long run.
    &[0u64; 32],
    // All ones.
    &[u64::MAX; 16],
    // Mixed: 16 zeros then 8 MAX.
    &[
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0, //
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0, //
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
    ],
];

fn assert_backend_matches_scalar(backend: unsafe fn(&[u64]) -> usize) {
    for &src in CASES {
        let expected = scalar::scan(src);
        // SAFETY: the backend requires the corresponding target feature, but
        // the test is only compiled when that feature is enabled.
        let actual = unsafe { backend(src) };

        assert_eq!(actual, expected, "src = {src:?}");
    }
}

// ---------------------------------------------------------------------------
// AVX2
// ---------------------------------------------------------------------------

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_matches_scalar() {
    assert_backend_matches_scalar(avx2::scan);
}

// ---------------------------------------------------------------------------
// SSE2
// ---------------------------------------------------------------------------

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2",
    not(target_feature = "avx2")
))]
#[test]
fn sse2_matches_scalar() {
    assert_backend_matches_scalar(sse2::scan);
}

// ---------------------------------------------------------------------------
// NEON
// ---------------------------------------------------------------------------

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar() {
    assert_backend_matches_scalar(neon::scan);
}
