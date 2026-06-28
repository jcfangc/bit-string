//! Verify that the SIMD pre-filter produces correct candidate positions
//! against the scalar oracle for random inputs.

use alloc::vec::Vec;

use proptest::collection::vec;
use proptest::prelude::*;

use crate::BitString;
use crate::WORD_BITS;
use crate::low_mask;

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
    fn find_matches_brute_force(
        h_bools in vec(any::<bool>(), 0..=256),
        n_bools in vec(any::<bool>(), 1..=64),
    ) {
        let haystack = BitString::from_bool_iter(h_bools);
        let needle = BitString::from_bool_iter(n_bools);
        let haystack_len = haystack.bit_len();
        let needle_len = needle.bit_len();

        let result = haystack.find_str(needle.as_bit_str());

        // Brute-force reference.
        let mut expected = None;
        let max_pos = haystack_len.saturating_sub(needle_len);
        for pos in 0..=max_pos {
            let mut ok = true;
            for j in 0..needle_len {
                let h = haystack.get(pos + j);
                let n = needle.get(j);
                if h != n {
                    ok = false;
                    break;
                }
            }
            if ok {
                expected = Some(pos);
                break;
            }
        }

        assert_eq!(result, expected,
            "find mismatch: haystack_len={haystack_len}, needle_len={needle_len}");
    }

    #[test]
    fn pre_filter_candidates_cover_all_true_matches(
        h_bools in vec(any::<bool>(), 64..=256),
        n_bools in vec(any::<bool>(), 1..=64),
    ) {
        let haystack = BitString::from_bool_iter(h_bools);
        let needle = BitString::from_bool_iter(n_bools);
        let haystack_len = haystack.bit_len();
        let needle_len = needle.bit_len();

        let haystack_words = haystack.words();
        let needle_words = needle.words();
        let needle_first = needle_words[0];
        let needle_mask = low_mask(needle_len.min(WORD_BITS));
        let last_start = haystack_len.saturating_sub(needle_len);

        // Collect scalar pre-filter candidates.
        let mut candidates = Vec::new();
        for shift in 0..WORD_BITS {
            for i in 0..haystack_words.len() {
                let pos = i * WORD_BITS + shift;
                if pos > last_start { break; }
                let window = if shift == 0 {
                    haystack_words[i]
                } else {
                    let w0 = haystack_words[i];
                    let w1 = haystack_words.get(i + 1).copied().unwrap_or(0);
                    (w0 >> shift) | (w1 << (WORD_BITS - shift))
                };
                if (window & needle_mask) == needle_first {
                    candidates.push(pos);
                }
            }
        }

        // Every true match must be in the candidates.
        for pos in 0..=last_start {
            let mut ok = true;
            for j in 0..needle_len {
                if haystack.get(pos + j) != needle.get(j) {
                    ok = false;
                    break;
                }
            }
            if ok {
                assert!(
                    candidates.contains(&pos),
                    "true match at {pos} missing from pre-filter candidates"
                );
            }
        }
    }
}
