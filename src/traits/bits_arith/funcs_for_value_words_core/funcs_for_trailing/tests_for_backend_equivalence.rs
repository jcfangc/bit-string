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

fn cases<const FILL: u64>() -> &'static [&'static [u64]] {
    if FILL == 0 { CASES_ZERO } else { CASES_ONE }
}

fn fill_val<const FILL: u64>() -> u64 {
    FILL
}

fn assert_backend_matches_scalar<const FILL: u64>(backend: Backend) {
    for &src in cases::<FILL>() {
        let expected = scalar::scan_rev::<FILL>(src);
        let actual = unsafe { backend(src) };
        assert_eq!(actual, expected, "fill={} src={src:?}", fill_val::<FILL>());
    }
}

fn run_random<const FILL: u64>(backend: Backend) {
    let fill = fill_val::<FILL>();
    for run in [0, 1, 3, 5, 7, 9, 15, 16, 17, 31] {
        let mut v = vec![fill; run];
        for _ in 0..16 {
            v.push(if FILL == 0 { u64::MAX } else { 0 });
        }
        let expected = scalar::scan_rev::<FILL>(&v);
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
    assert_backend_matches_scalar::<0>(avx2::scan_rev::<0>);
    run_random::<0>(avx2::scan_rev::<0>);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_matches_scalar_ones() {
    assert_backend_matches_scalar::<{ u64::MAX }>(avx2::scan_rev::<{ u64::MAX }>);
    run_random::<{ u64::MAX }>(avx2::scan_rev::<{ u64::MAX }>);
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
    assert_backend_matches_scalar::<0>(sse41::scan_rev::<0>);
    run_random::<0>(sse41::scan_rev::<0>);
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1",
    not(target_feature = "avx2")
))]
#[test]
fn sse41_matches_scalar_ones() {
    assert_backend_matches_scalar::<{ u64::MAX }>(sse41::scan_rev::<{ u64::MAX }>);
    run_random::<{ u64::MAX }>(sse41::scan_rev::<{ u64::MAX }>);
}

// ---------------------------------------------------------------------------
// NEON
// ---------------------------------------------------------------------------

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar_zeros() {
    assert_backend_matches_scalar::<0>(neon::scan_rev::<0>);
    run_random::<0>(neon::scan_rev::<0>);
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_matches_scalar_ones() {
    assert_backend_matches_scalar::<{ u64::MAX }>(neon::scan_rev::<{ u64::MAX }>);
    run_random::<{ u64::MAX }>(neon::scan_rev::<{ u64::MAX }>);
}
