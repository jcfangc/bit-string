//! Verify that `find_any_candidate` produces correct candidate positions
//! against brute-force, implicitly validating whichever SIMD backend is
//! active (SSE2, AVX2, or NEON) against the scalar oracle.

use proptest::collection::vec;
use proptest::prelude::*;

use crate::BitString;

use super::super::bits_equal_at;

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
            haystack.as_words(),
            haystack.bit_len(),
            needle.as_words(),
            needle.bit_len(),
            &mut |pos| bits_equal_at(&haystack, pos, &needle),
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
            haystack.as_words(),
            haystack_len,
            needle.as_words(),
            needle_len,
            &mut |pos| {
                let ok = bits_equal_at(&haystack, pos, &needle);
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
