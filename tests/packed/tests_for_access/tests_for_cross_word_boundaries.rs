use super::*;
use proptest::prelude::*;

proptest! {
#[test]
    fn three_and_seven_bit_access_and_slice_cross_word_boundaries(
        left3 in prop::collection::vec(0u8..=7, 22..=128),
        left7 in prop::collection::vec(0u8..=127, 10..=128),
        start in 0usize..160,
        len in 1usize..80,
    ) {
        assert_access_slice::<Oct, 3>(&left3, start, len, oct);
        assert_access_slice::<WideCode, 7>(&left7, start, len, wide);
    }
}
