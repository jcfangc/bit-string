use super::{Oct, WideCode, assert_edits, packed, symbol};
use bit_string::traits::PackedChar;
use int_intervals::UsizeCO;
use proptest::prelude::*;

proptest! {
    #[test]
    fn packed_edits_match_vec_splice_semantics(
        initial in prop::collection::vec(0u8..=2, 0..=24),
        replacement in prop::collection::vec(0u8..=2, 0..=12),
        insert_index in 0usize..32,
        remove_index in 0usize..32,
        replace_start in 0usize..32,
    ) {
        let mut string = packed(&initial);
        let mut oracle = initial.clone();

        let oracle_insert_index = insert_index.min(oracle.len());
        let inserted = symbol(2);
        string.insert(insert_index, inserted);
        oracle.insert(oracle_insert_index, 2);

        if !oracle.is_empty() {
            let remove_index = remove_index.min(oracle.len() - 1);
            prop_assert_eq!(string.remove(remove_index).code(), oracle.remove(remove_index));
        }

        let oracle_replace_start = replace_start.min(oracle.len());
        let replace_end = oracle_replace_start
            .saturating_add(replacement.len())
            .min(oracle.len());
        string.replace_assign(replace_start, &packed(&replacement));
        oracle.splice(oracle_replace_start..replace_end, replacement.iter().copied());
        prop_assert_eq!(
            string.to_vec().iter().copied().map(|value| value.code()).collect::<Vec<_>>(),
            oracle.clone(),
        );

        let drain_len = insert_index.max(1);
        let oracle_drain_start = remove_index.min(oracle.len());
        let oracle_drain_end = remove_index
            .saturating_add(drain_len)
            .min(oracle.len())
            .max(oracle_drain_start);
        string.drain_interval_assign(
            UsizeCO::checked_from_start_len(remove_index, drain_len).unwrap(),
        );
        oracle.drain(oracle_drain_start..oracle_drain_end);
        prop_assert_eq!(
            string.to_vec().iter().copied().map(|value| value.code()).collect::<Vec<_>>(),
            oracle.clone(),
        );
    }

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
