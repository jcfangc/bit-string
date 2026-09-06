use super::{Oct, WideCode, assert_edits};
use proptest::prelude::*;

proptest! {
#[test]
    fn three_and_seven_bit_edits_cross_word_boundaries(
        initial3 in prop::collection::vec(0u8..=7, 22..=128),
        replacement3 in prop::collection::vec(0u8..=7, 0..=64),
        initial7 in prop::collection::vec(0u8..=127, 10..=128),
        replacement7 in prop::collection::vec(0u8..=127, 0..=32),
        insert_index in 0usize..160,
        remove_index in 0usize..160,
        replace_start in 0usize..160,
    ) {
        assert_edits::<Oct, 3>(&initial3, &replacement3, insert_index, remove_index, replace_start, 2, super::super::oct);
        assert_edits::<WideCode, 7>(&initial7, &replacement7, insert_index, remove_index, replace_start, 2, super::super::wide);
    }
}
