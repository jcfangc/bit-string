use super::{Oct, WideCode, assert_matching, matches_at, oct, packed, wide};
use proptest::prelude::*;

proptest! {
    #[test]
    fn packed_matching_matches_vec_sliding_windows(
        haystack in prop::collection::vec(0u8..=2, 0..=32),
        needle in prop::collection::vec(0u8..=2, 0..=16),
    ) {
        let haystack_string = packed(&haystack);
        let needle_string = packed(&needle);
        let haystack_view = haystack_string.as_packed_str();
        let needle_view = needle_string.as_packed_str();
        let max_start = haystack.len().saturating_sub(needle.len());
        let expected_find = (0..=max_start).find(|&start| matches_at(&haystack, start, &needle));
        let expected_rfind = (0..=max_start).rev().find(|&start| matches_at(&haystack, start, &needle));

        prop_assert_eq!(haystack_view.find(needle_view), expected_find);
        prop_assert_eq!(haystack_view.rfind(needle_view), expected_rfind);
        prop_assert_eq!(haystack_view.contains(needle_view), expected_find.is_some());
    }

    #[test]
    fn three_and_seven_bit_matching_crosses_word_boundaries(
        haystack3 in prop::collection::vec(0u8..=7, 22..=128),
        needle3 in prop::collection::vec(0u8..=7, 0..=32),
        haystack7 in prop::collection::vec(0u8..=127, 10..=128),
        needle7 in prop::collection::vec(0u8..=127, 0..=16),
    ) {
        assert_matching::<Oct, 3>(&haystack3, &needle3, oct);
        assert_matching::<WideCode, 7>(&haystack7, &needle7, wide);
    }
}
