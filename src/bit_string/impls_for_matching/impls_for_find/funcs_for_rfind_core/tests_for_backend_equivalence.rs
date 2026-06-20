//! Verify that `find_last_word` produces correct rightmost matches
//! against brute-force, implicitly validating whichever SIMD backend is
//! active (SSE2, AVX2, or NEON) against the scalar oracle.

use alloc::vec::Vec;

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
    fn rfind_matches_brute_force_rightmost(
        h_bools in vec(any::<bool>(), 64..=256),
        n_bools in vec(any::<bool>(), 1..=64),
    ) {
        let haystack = BitString::from_bool_iter(h_bools);
        let needle = BitString::from_bool_iter(n_bools);

        let result = super::find_last_word(
            haystack.as_words(),
            haystack.bit_len(),
            needle.as_words(),
            needle.bit_len(),
            &mut |pos| bits_equal_at(&haystack, pos, &needle),
        );

        // Brute-force: find rightmost match.
        let mut expected = None;
        let max_pos = haystack.bit_len().saturating_sub(needle.bit_len());
        for pos in (0..=max_pos).rev() {
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
            result, expected,
            "rfind mismatch"
        );
    }

    #[test]
    fn rfind_returns_none_only_when_no_match_exists(
        h_bools in vec(any::<bool>(), 64..=256),
        n_bools in vec(any::<bool>(), 1..=64),
    ) {
        let haystack = BitString::from_bool_iter(h_bools);
        let needle = BitString::from_bool_iter(n_bools);

        let result = super::find_last_word(
            haystack.as_words(),
            haystack.bit_len(),
            needle.as_words(),
            needle.bit_len(),
            &mut |pos| bits_equal_at(&haystack, pos, &needle),
        );

        let max_pos = haystack.bit_len().saturating_sub(needle.bit_len());
        let mut any = false;
        for pos in 0..=max_pos {
            let mut ok = true;
            for j in 0..needle.bit_len() {
                if haystack.get(pos + j) != needle.get(j) {
                    ok = false;
                    break;
                }
            }
            if ok { any = true; break; }
        }

        assert_eq!(result.is_some(), any, "presence mismatch");
    }
}
