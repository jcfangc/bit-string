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

type Backend = unsafe fn(&[u64]) -> usize;

fn cases<const FILL_ONES: bool>() -> &'static [&'static [u64]] {
    if FILL_ONES { CASES_ONE } else { CASES_ZERO }
}

fn fill_val<const FILL_ONES: bool>() -> u64 {
    if FILL_ONES { !0u64 } else { 0 }
}

fn assert_backend_matches_scalar<const FILL_ONES: bool>(backend: Backend) {
    for &src in cases::<FILL_ONES>() {
        let expected = scalar::scan_rev::<FILL_ONES>(src);
        let actual = unsafe { backend(src) };
        assert_eq!(
            actual,
            expected,
            "fill={} src={src:?}",
            fill_val::<FILL_ONES>()
        );
    }
}

fn run_random<const FILL_ONES: bool>(backend: Backend) {
    let fill = fill_val::<FILL_ONES>();
    for run in [0, 1, 3, 5, 7, 9, 15, 16, 17, 31] {
        let mut v = vec![fill; run];
        for _ in 0..16 {
            v.push(if FILL_ONES { 0 } else { u64::MAX });
        }
        let expected = scalar::scan_rev::<FILL_ONES>(&v);
        let actual = unsafe { backend(&v) };
        assert_eq!(actual, expected, "fill={fill} run={run}");
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
    assert_backend_matches_scalar::<false>(avx2::scan_rev::<false>);
    run_random::<false>(avx2::scan_rev::<false>);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_matches_scalar_ones() {
    assert_backend_matches_scalar::<true>(avx2::scan_rev::<true>);
    run_random::<true>(avx2::scan_rev::<true>);
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
    assert_backend_matches_scalar::<false>(sse41::scan_rev::<false>);
    run_random::<false>(sse41::scan_rev::<false>);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1",
    not(target_feature = "avx2")
))]
#[test]
fn sse41_matches_scalar_ones() {
    assert_backend_matches_scalar::<true>(sse41::scan_rev::<true>);
    run_random::<true>(sse41::scan_rev::<true>);
}

// ---------------------------------------------------------------------------
// NEON
// ---------------------------------------------------------------------------

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar_zeros() {
    assert_backend_matches_scalar::<false>(neon::scan_rev::<false>);
    run_random::<false>(neon::scan_rev::<false>);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar_ones() {
    assert_backend_matches_scalar::<true>(neon::scan_rev::<true>);
    run_random::<true>(neon::scan_rev::<true>);
}
