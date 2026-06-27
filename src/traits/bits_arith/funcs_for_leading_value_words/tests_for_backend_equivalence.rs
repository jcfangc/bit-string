use alloc::vec;

use super::*;

const CASES_ZERO: &[&[u64]] = &[
    &[],
    &[0],
    &[u64::MAX],
    &[1],
    &[0, u64::MAX],
    &[0, 0, u64::MAX],
    &[0, 0, 0, 0, u64::MAX],
    &[0, 0, 0, 0, 0, 0, 0, u64::MAX],
    &[0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA],
    &[0, 1, u64::MAX, 0x0123_4567_89AB_CDEF],
    &[0u64; 32],
    &[u64::MAX; 16],
];

const CASES_ONE: &[&[u64]] = &[
    &[],
    &[u64::MAX],
    &[0],
    &[u64::MAX - 1],
    &[u64::MAX, 0],
    &[u64::MAX, u64::MAX, 0],
    &[u64::MAX, u64::MAX, u64::MAX, u64::MAX, 0],
    &[
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        0,
    ],
    &[0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA],
    &[u64::MAX, 1, 0, 0x0123_4567_89AB_CDEF],
    &[u64::MAX; 32],
    &[0u64; 16],
];

fn assert_backend_matches_scalar(backend: unsafe fn(&[u64], u64) -> usize, fill: u64) {
    let cases: &[&[u64]] = if fill == 0 { CASES_ZERO } else { CASES_ONE };

    for &src in cases {
        let expected = scalar::scan(src, fill);
        // SAFETY: the backend requires the corresponding target feature, but
        // the test is only compiled when that feature is enabled.
        let actual = unsafe { backend(src, fill) };

        assert_eq!(actual, expected, "fill=0x{fill:x} src={src:?}");
    }
}

// Also test with random-looking data.
fn run_random(backend: unsafe fn(&[u64], u64) -> usize, fill: u64) {
    for run in [0, 1, 3, 5, 7, 9, 15, 16, 17, 31] {
        let mut v = vec![fill; run];
        for _ in 0..16 {
            v.push(if fill == 0 { u64::MAX } else { 0 });
        }
        let expected = scalar::scan(&v, fill);
        let actual = unsafe { backend(&v, fill) };
        assert_eq!(actual, expected, "fill=0x{fill:x} run={run}");
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
fn avx2_matches_scalar_zeros() {
    assert_backend_matches_scalar(avx2::scan, 0);
    run_random(avx2::scan, 0);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_matches_scalar_ones() {
    assert_backend_matches_scalar(avx2::scan, !0);
    run_random(avx2::scan, !0);
}

// ---------------------------------------------------------------------------
// SSE4.1
// ---------------------------------------------------------------------------

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1",
    not(target_feature = "avx2")
))]
#[test]
fn sse41_matches_scalar_zeros() {
    assert_backend_matches_scalar(sse41::scan, 0);
    run_random(sse41::scan, 0);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1",
    not(target_feature = "avx2")
))]
#[test]
fn sse41_matches_scalar_ones() {
    assert_backend_matches_scalar(sse41::scan, !0);
    run_random(sse41::scan, !0);
}

// ---------------------------------------------------------------------------
// NEON
// ---------------------------------------------------------------------------

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar_zeros() {
    assert_backend_matches_scalar(neon::scan, 0);
    run_random(neon::scan, 0);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar_ones() {
    assert_backend_matches_scalar(neon::scan, !0);
    run_random(neon::scan, !0);
}

// ---------------------------------------------------------------------------
// Trailing scan (reverse) — AVX2 / SSE4.1 / NEON
// ---------------------------------------------------------------------------

fn assert_trailing_backend_matches_scalar(backend: unsafe fn(&[u64], u64) -> usize, fill: u64) {
    let cases: &[&[u64]] = if fill == 0 { CASES_ZERO } else { CASES_ONE };

    for &src in cases {
        let expected = scalar::scan_rev(src, fill);
        let actual = unsafe { backend(src, fill) };
        assert_eq!(actual, expected, "fill=0x{fill:x} src={src:?}");
    }
}

fn run_random_trailing(backend: unsafe fn(&[u64], u64) -> usize, fill: u64) {
    for run in [0, 1, 3, 5, 7, 9, 15, 16, 17, 31] {
        let mut v = vec![fill; run];
        for _ in 0..16 {
            v.push(if fill == 0 { u64::MAX } else { 0 });
        }
        let expected = scalar::scan_rev(&v, fill);
        let actual = unsafe { backend(&v, fill) };
        assert_eq!(actual, expected, "fill=0x{fill:x} run={run}");
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_trailing_matches_scalar_zeros() {
    assert_trailing_backend_matches_scalar(avx2::scan_rev, 0);
    run_random_trailing(avx2::scan_rev, 0);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_trailing_matches_scalar_ones() {
    assert_trailing_backend_matches_scalar(avx2::scan_rev, !0);
    run_random_trailing(avx2::scan_rev, !0);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1",
    not(target_feature = "avx2")
))]
#[test]
fn sse41_trailing_matches_scalar_zeros() {
    assert_trailing_backend_matches_scalar(sse41::scan_rev, 0);
    run_random_trailing(sse41::scan_rev, 0);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1",
    not(target_feature = "avx2")
))]
#[test]
fn sse41_trailing_matches_scalar_ones() {
    assert_trailing_backend_matches_scalar(sse41::scan_rev, !0);
    run_random_trailing(sse41::scan_rev, !0);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_trailing_matches_scalar_zeros() {
    assert_trailing_backend_matches_scalar(neon::scan_rev, 0);
    run_random_trailing(neon::scan_rev, 0);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_trailing_matches_scalar_ones() {
    assert_trailing_backend_matches_scalar(neon::scan_rev, !0);
    run_random_trailing(neon::scan_rev, !0);
}
