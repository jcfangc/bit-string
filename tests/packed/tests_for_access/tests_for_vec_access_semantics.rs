use int_intervals::UsizeCO;
use proptest::prelude::*;

proptest! {
#[test]
    fn packed_access_and_slice_match_vec_oracles(
        left in prop::collection::vec(0u8..=2, 0..=32),
        start in 0usize..40,
        len in 1usize..40,
    ) {
        let string = super::packed(&left);
        prop_assert_eq!(string.char_len(), left.len());
        let view = string.as_packed_str();
        for index in 0..=left.len() + 1 {
            prop_assert_eq!(string.get(index), left.get(index).copied().map(super::symbol));
            prop_assert_eq!(view.get(index), left.get(index).copied().map(super::symbol));
        }

        let oracle_start = start.min(left.len());
        let oracle_end = start.saturating_add(len).min(left.len()).max(oracle_start);
        let slice = string.slice(UsizeCO::checked_from_start_len(start, len).unwrap());
        prop_assert_eq!(slice.to_vec(), left[oracle_start..oracle_end].iter().copied().map(super::symbol).collect::<Vec<_>>());

    }
}
