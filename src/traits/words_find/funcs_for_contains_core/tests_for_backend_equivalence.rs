//! Verify that `find_any_candidate` produces correct candidate positions
//! against brute-force, implicitly validating whichever SIMD backend is
//! active (SSE4.1, AVX2, or NEON) against the scalar oracle.

use proptest::collection::vec;
use proptest::prelude::*;

use crate::BitString;

#[test]
fn find_any_handles_candidates_at_the_end_of_the_backing_slice() {
    // Exercise the SIMD threshold with no guard word after the slice. Shifted
    // vector windows need one extra source word, so their final lane must use
    // the scalar tail when that word is not present.
    for word_len in 1..=5 {
        let haystack = alloc::vec![0u64; word_len];
        assert_eq!(
            super::find_any_candidate(&haystack, word_len * crate::WORD_BITS, &[1], 1, &mut |_| {
                true
            },),
            None,
            "word_len={word_len}"
        );
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_shifted_window_does_not_read_past_four_word_slice() {
    let haystack = [0u64; 4];
    // SAFETY: this test is compiled only when AVX2 is enabled for the target.
    assert_eq!(
        unsafe {
            super::avx2::find_any(
                &haystack,
                1,
                1,
                haystack.len() * crate::WORD_BITS - 1,
                haystack.len(),
                &mut |_| true,
            )
        },
        None
    );
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1",
    not(target_feature = "avx2")
))]
#[test]
fn sse41_shifted_window_does_not_read_past_two_word_slice() {
    let haystack = [0u64; 2];
    // SAFETY: this test is compiled only when SSE4.1 is enabled for the target.
    assert_eq!(
        unsafe {
            super::sse41::find_any(
                &haystack,
                1,
                1,
                haystack.len() * crate::WORD_BITS - 1,
                haystack.len(),
                &mut |_| true,
            )
        },
        None
    );
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[test]
fn neon_shift_zero_does_not_read_past_odd_word_slice() {
    let haystack = [0u64; 3];
    // SAFETY: NEON is enabled for all supported AArch64 targets.
    assert_eq!(
        unsafe {
            super::neon::find_any(
                &haystack,
                1,
                1,
                haystack.len() * crate::WORD_BITS - 1,
                haystack.len(),
                &mut |_| true,
            )
        },
        None
    );
}

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 512,
        max_shrink_iters: 128,
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn find_any_matches_brute_force(
        h_bools in vec(any::<bool>(), 64..=256),
        n_bools in vec(any::<bool>(), 1..=64),
    ) {
        let haystack = BitString::from_bool_iter(h_bools);
        let needle = BitString::from_bool_iter(n_bools);

        let result = super::find_any_candidate(
            haystack.words(),
            haystack.bit_len(),
            needle.words(),
            needle.bit_len(),
            &mut |pos| haystack.as_bit_str().bits_equal_at_inner::<false, false>(pos, needle.as_bit_str()),
        );

        // Brute-force reference: find any match.
        let mut expected = None;
        let max_pos = haystack.bit_len().saturating_sub(needle.bit_len());
        for pos in 0..=max_pos {
            let mut ok = true;
            for j in 0..needle.bit_len() {
                if haystack.get(pos + j) != needle.get(j) {
                    ok = false;
                    break;
                }
            }
            if ok {
                expected = Some(pos);
                break;
            }
        }

        assert_eq!(
            result.is_some(),
            expected.is_some(),
            "find_any_candidate presence mismatch"
        );
    }

    #[test]
    fn find_any_candidates_cover_all_true_matches(
        h_bools in vec(any::<bool>(), 64..=256),
        n_bools in vec(any::<bool>(), 1..=64),
    ) {
        let haystack = BitString::from_bool_iter(h_bools);
        let needle = BitString::from_bool_iter(n_bools);
        let haystack_len = haystack.bit_len();
        let needle_len = needle.bit_len();

        let mut any_found = false;
        let _ = super::find_any_candidate(
            haystack.words(),
            haystack_len,
            needle.words(),
            needle_len,
            &mut |pos| {
                let ok = haystack.as_bit_str().bits_equal_at_inner::<false, false>(pos, needle.as_bit_str());
                if ok { any_found = true; }
                ok
            },
        );

        // If brute-force finds a match, find_any_candidate must find one too.
        let max_pos = haystack_len.saturating_sub(needle_len);
        let mut brute_has_match = false;
        for pos in 0..=max_pos {
            let mut ok = true;
            for j in 0..needle_len {
                if haystack.get(pos + j) != needle.get(j) {
                    ok = false;
                    break;
                }
            }
            if ok {
                brute_has_match = true;
                break;
            }
        }

        assert_eq!(any_found, brute_has_match,
            "find_any_candidate presence check failed");
    }
}
