use super::super::symbol;
use super::*;
use proptest::prelude::*;

#[test]
fn packed_remove_returns_deleted_code_and_preserves_the_rest() {
    let mut oct_codes: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let mut oct_string = PackedString::from_chars(oct_codes.clone());
    for index in [0, 10, 19] {
        let expected = oct_codes.remove(index);
        assert_eq!(oct_string.remove(index), expected);
        assert_eq!(oct_string.to_vec(), oct_codes);
        assert_eq!(oct_string.char_len(), oct_codes.len());
        assert_eq!(oct_string.bits().bit_len(), oct_codes.len() * 3);
    }

    let mut wide_codes: Vec<_> = (0..10)
        .map(|index| {
            if index == 9 {
                WideCode(127)
            } else {
                WideCode((index * 11) as u8 % 128)
            }
        })
        .collect();
    let mut wide_string = PackedString::from_chars(wide_codes.clone());
    assert_eq!(wide_string.remove(9), WideCode(127));
    wide_codes.pop();
    assert_eq!(wide_string.to_vec(), wide_codes);
    assert_eq!(wide_string.bits().bit_len(), wide_codes.len() * 7);

    let mut bytes = PackedString::from_chars([
        super::SparseByte::Zero,
        super::SparseByte::Maximum,
        super::SparseByte::Middle,
    ]);
    assert_eq!(bytes.remove(1), super::SparseByte::Maximum);
    assert_eq!(
        bytes.to_vec(),
        vec![super::SparseByte::Zero, super::SparseByte::Middle]
    );

    let mut empty = PackedString::<Oct, 3>::new();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| empty.remove(0))).is_err());
    let mut singleton = PackedString::from_chars([Oct::V0]);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| singleton.remove(1))).is_err()
    );
    assert_eq!(singleton.to_vec(), vec![Oct::V0]);
}

#[test]
fn packed_retain_matches_vec_order_and_predicate_calls() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let mut string = PackedString::from_chars(original.clone());
    let mut seen = Vec::new();
    string.retain(|character| {
        seen.push(character);
        character.code() % 2 == 0
    });

    let mut oracle = original.clone();
    oracle.retain(|character| character.code() % 2 == 0);
    assert_eq!(string.to_vec(), oracle);
    assert_eq!(seen, original);
    assert_eq!(string.char_len(), oracle.len());
    assert_eq!(string.bits().bit_len(), oracle.len() * 3);

    let mut all = PackedString::from_chars([Oct::V7; 22]);
    all.retain(|_| true);
    assert_eq!(all.to_vec(), vec![Oct::V7; 22]);

    let mut none = PackedString::from_chars([Oct::V0; 22]);
    none.retain(|_| false);
    assert!(none.is_empty());
    assert!(none.bits().words().is_empty());

    let wide_values: Vec<_> = (0..10)
        .map(|index| WideCode((index * 11) as u8 % 128))
        .collect();
    let mut wide = PackedString::from_chars(wide_values.clone());
    wide.retain(|character| character.code() >= 64);
    let expected_wide: Vec<_> = wide_values
        .into_iter()
        .filter(|character| character.code() >= 64)
        .collect();
    assert_eq!(wide.to_vec(), expected_wide);
    assert_eq!(wide.bits().bit_len(), expected_wide.len() * 7);

    let mut bytes = PackedString::from_chars([
        super::SparseByte::Zero,
        super::SparseByte::Maximum,
        super::SparseByte::Middle,
    ]);
    bytes.retain(|character| character.code() != 3);
    assert_eq!(
        bytes.to_vec(),
        vec![super::SparseByte::Zero, super::SparseByte::Maximum]
    );
}

#[test]
fn packed_insert_matches_vec_insert_and_clamps_out_of_bounds_indices() {
    let mut string = PackedString::from_chars((0..22).map(|index| super::oct(index as u8 % 8)));
    let mut oracle = string.to_vec();

    for (index, character) in [(0, Oct::V7), (10, Oct::V0), (usize::MAX, Oct::V5)] {
        let insertion_index = index.min(oracle.len());
        string.insert(index, character);
        oracle.insert(insertion_index, character);
        assert_eq!(string.to_vec(), oracle);
        assert_eq!(string.char_len(), oracle.len());
        assert_eq!(string.bits().bit_len(), oracle.len() * 3);
    }

    let mut wide =
        PackedString::from_chars((0..10).map(|index| WideCode((index * 13) as u8 % 128)));
    let mut wide_oracle = wide.to_vec();
    let insertion_index = 9;
    wide.insert(insertion_index, WideCode(127));
    wide_oracle.insert(insertion_index, WideCode(127));
    assert_eq!(wide.to_vec(), wide_oracle);
    assert_eq!(wide.bits().bit_len(), wide_oracle.len() * 7);

    let mut empty = PackedString::<Oct, 3>::new();
    empty.insert(usize::MAX, Oct::V0);
    assert_eq!(empty.to_vec(), vec![Oct::V0]);
    assert_eq!(empty.bits().bit_len(), 3);
}

#[test]
fn packed_insert_packed_string_splices_codes_and_preserves_source() {
    let original_codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let replacement_codes = vec![7, 0, 3];
    let mut string = packed_as::<Oct, 3>(&original_codes, super::oct);
    let replacement = packed_as::<Oct, 3>(&replacement_codes, super::oct);
    let mut oracle = original_codes.clone();

    string.insert_packed_string(9, &replacement);
    oracle.splice(9..9, replacement_codes.iter().copied());
    assert_eq!(
        string
            .to_vec()
            .iter()
            .map(|value| value.code())
            .collect::<Vec<_>>(),
        oracle
    );
    assert_eq!(string.bits().bit_len(), oracle.len() * 3);
    assert_eq!(
        replacement.to_vec(),
        replacement_codes
            .iter()
            .map(|&code| super::oct(code))
            .collect::<Vec<_>>()
    );

    string.insert_packed_string(usize::MAX, &replacement);
    oracle.extend(replacement_codes.iter().copied());
    assert_eq!(
        string
            .to_vec()
            .iter()
            .map(|value| value.code())
            .collect::<Vec<_>>(),
        oracle
    );
    assert_eq!(string.bits().bit_len(), oracle.len() * 3);

    let empty = PackedString::<Oct, 3>::new();
    let before = string.clone();
    string.insert_packed_string(4, &empty);
    assert_eq!(string, before);

    let wide = packed_as::<WideCode, 7>(&[1, 64, 127], super::wide);
    let mut wide_target = packed_as::<WideCode, 7>(&[0; 10], super::wide);
    wide_target.insert_packed_string(8, &wide);
    assert_eq!(
        wide_target.to_vec(),
        vec![
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(0),
            WideCode(1),
            WideCode(64),
            WideCode(127),
            WideCode(0),
            WideCode(0),
        ]
    );
    assert_eq!(wide_target.bits().bit_len(), 13 * 7);
}

#[test]
fn packed_drain_interval_removes_clamped_character_ranges_without_mutating_source() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let string = PackedString::from_chars(original.clone());

    let interval = UsizeCO::checked_from_start_len(9, 5).unwrap();
    let drained = string.drain_interval(interval);
    let mut expected = original.clone();
    expected.drain(9..14);
    assert_eq!(drained.to_vec(), expected);
    assert_eq!(drained.char_len(), expected.len());
    assert_eq!(drained.bits().bit_len(), expected.len() * 3);
    assert_eq!(string.to_vec(), original);

    let tail = string.drain_interval(UsizeCO::checked_from_start_len(20, 9).unwrap());
    assert_eq!(tail.to_vec(), original[..20].to_vec());
    assert_eq!(tail.bits().bit_len(), 20 * 3);

    let out_of_bounds = string.drain_interval(UsizeCO::checked_from_start_len(99, 4).unwrap());
    assert_eq!(out_of_bounds.to_vec(), original);
    assert_eq!(out_of_bounds.bits().words(), string.bits().words());

    let wide_values: Vec<_> = (0..10)
        .map(|index| WideCode((index * 13) as u8 % 128))
        .collect();
    let wide = PackedString::from_chars(wide_values.clone());
    let wide_drained = wide.drain_interval(UsizeCO::checked_from_start_len(7, 2).unwrap());
    let mut wide_expected = wide_values.clone();
    wide_expected.drain(7..9);
    assert_eq!(wide_drained.to_vec(), wide_expected);
    assert_eq!(wide_drained.bits().bit_len(), wide_expected.len() * 7);
}

#[test]
fn packed_drain_interval_assign_matches_non_mutating_drain() {
    let original: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let interval = UsizeCO::checked_from_start_len(9, 5).unwrap();
    let expected = PackedString::from_chars(original.clone()).drain_interval(interval);
    let mut assigned = PackedString::from_chars(original);
    assigned.drain_interval_assign(interval);
    assert_eq!(assigned.to_vec(), expected.to_vec());
    assert_eq!(assigned.bits().words(), expected.bits().words());
    assert_eq!(assigned.bits().bit_len(), assigned.char_len() * 3);

    let mut unchanged = assigned.clone();
    let before = unchanged.bits().clone();
    unchanged.drain_interval_assign(UsizeCO::checked_from_start_len(99, 4).unwrap());
    assert_eq!(unchanged.bits().words(), before.words());
    assert_eq!(unchanged.to_vec(), assigned.to_vec());

    let mut empty = PackedString::<Oct, 3>::new();
    empty.drain_interval_assign(UsizeCO::checked_from_start_len(0, 1).unwrap());
    assert!(empty.is_empty());

    let wide_values: Vec<_> = (0..10)
        .map(|index| WideCode((index * 13) as u8 % 128))
        .collect();
    let wide_interval = UsizeCO::checked_from_start_len(7, 2).unwrap();
    let wide_expected = PackedString::from_chars(wide_values.clone()).drain_interval(wide_interval);
    let mut wide = PackedString::from_chars(wide_values);
    wide.drain_interval_assign(wide_interval);
    assert_eq!(wide.to_vec(), wide_expected.to_vec());
    assert_eq!(wide.bits().bit_len(), wide.char_len() * 7);
}

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
}
