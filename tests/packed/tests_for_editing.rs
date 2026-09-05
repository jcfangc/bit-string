use super::{Oct, WideCode, packed, packed_as, symbol};
use bit_string::{BitString, PackedString, traits::PackedChar};
use int_intervals::UsizeCO;
use proptest::prelude::*;

fn assert_edits<C, const BITS: u8>(
    initial: &[u8],
    replacement: &[u8],
    insert_index: usize,
    remove_index: usize,
    replace_start: usize,
    inserted_code: u8,
    decode: fn(u8) -> C,
) where
    C: PackedChar<BITS> + core::fmt::Debug,
{
    let mut string = packed_as(initial, decode);
    let mut oracle = initial.to_vec();

    let oracle_insert_index = insert_index.min(oracle.len());
    string.insert(insert_index, decode(inserted_code));
    oracle.insert(oracle_insert_index, inserted_code);

    if !oracle.is_empty() {
        let remove_index = remove_index.min(oracle.len() - 1);
        assert_eq!(
            string.remove(remove_index).code(),
            oracle.remove(remove_index)
        );
    }

    let oracle_replace_start = replace_start.min(oracle.len());
    let replace_end = oracle_replace_start
        .saturating_add(replacement.len())
        .min(oracle.len());
    string.replace_assign(replace_start, &packed_as(replacement, decode));
    oracle.splice(
        oracle_replace_start..replace_end,
        replacement.iter().copied(),
    );
    assert_eq!(
        string
            .to_vec()
            .iter()
            .copied()
            .map(|value| value.code())
            .collect::<Vec<_>>(),
        oracle
    );

    let drain_len = insert_index.max(1);
    let oracle_drain_start = remove_index.min(oracle.len());
    let oracle_drain_end = remove_index
        .saturating_add(drain_len)
        .min(oracle.len())
        .max(oracle_drain_start);
    string.drain_interval_assign(UsizeCO::checked_from_start_len(remove_index, drain_len).unwrap());
    oracle.drain(oracle_drain_start..oracle_drain_end);
    assert_eq!(
        string
            .to_vec()
            .iter()
            .copied()
            .map(|value| value.code())
            .collect::<Vec<_>>(),
        oracle
    );
}

#[test]
fn packed_out_of_bounds_intervals_preserve_empty_semantics() {
    let string = packed(&[0, 1, 2]);
    let out_of_bounds = UsizeCO::checked_from_start_len(99, 1).unwrap();

    assert!(string.slice(out_of_bounds).is_empty());

    let mut drained = string.clone();
    drained.drain_interval_assign(out_of_bounds);
    assert_eq!(drained, string);
}

#[test]
fn packed_clear_removes_all_codes_and_allows_reuse() {
    let mut empty = PackedString::<super::Symbol, 2>::new();
    empty.clear();
    empty.clear();
    assert!(empty.is_empty());
    assert_eq!(empty.char_len(), 0);
    assert_eq!(empty.bits().bit_len(), 0);
    assert!(empty.bits().words().is_empty());
    assert!(empty.to_vec().is_empty());
    assert_eq!(empty.get(0), None);
    assert_eq!(empty.first(), None);
    assert_eq!(empty.last(), None);
    assert_eq!(empty.as_packed_str().iter().next(), None);

    let mut zero = packed_as::<super::SparseByte, 8>(&[0], |code| match code {
        0 => super::SparseByte::Zero,
        _ => unreachable!(),
    });
    assert!(!zero.is_empty());
    assert_eq!(zero.bits().words(), &[0]);
    zero.clear();
    assert!(zero.is_empty());
    assert!(zero.bits().words().is_empty());

    let oct_codes: Vec<_> = (0..22).map(|index| index as u8 % 8).collect();
    let mut oct_string = packed_as::<Oct, 3>(&oct_codes, super::oct);
    oct_string.clear();
    assert_eq!(oct_string.char_len(), 0);
    assert_eq!(oct_string.bits().bit_len(), 0);
    assert!(oct_string.bits().words().is_empty());
    oct_string.push(Oct::V7);
    assert_eq!(oct_string.to_vec(), vec![Oct::V7]);
    assert_eq!(oct_string.bits().bit_len(), 3);

    let wide_codes: Vec<_> = (0..10).map(|index| (index * 11) as u8 % 128).collect();
    let mut wide_string = packed_as::<WideCode, 7>(&wide_codes, super::wide);
    wide_string.clear();
    wide_string.push(WideCode(127));
    assert_eq!(wide_string.to_vec(), vec![WideCode(127)]);
    assert_eq!(wide_string.bits().bit_len(), 7);
}

#[test]
fn packed_extend_copies_borrowed_codes_in_order() {
    let mut oct_string = packed_as::<Oct, 3>(&[Oct::V7.code()], super::oct);
    let mut oct_source: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();
    let mut oct_expected = oct_string.to_vec();
    oct_expected.extend(oct_source.iter().copied());
    oct_string.extend(oct_source.iter());
    assert_eq!(oct_string.to_vec(), oct_expected);
    assert_eq!(oct_string.char_len(), oct_expected.len());
    assert_eq!(oct_string.bits().bit_len(), oct_expected.len() * 3);

    oct_source[0] = Oct::V0;
    oct_source.clear();
    assert_eq!(oct_string.to_vec(), oct_expected);

    let mut wide_string = packed_as::<WideCode, 7>(&[126], super::wide);
    let mut wide_source: Vec<_> = (0..10)
        .map(|index| {
            if index == 9 {
                WideCode(127)
            } else {
                WideCode((index * 11) as u8 % 128)
            }
        })
        .collect();
    let mut wide_expected = wide_string.to_vec();
    wide_expected.extend(wide_source.iter().copied());
    wide_string.extend(wide_source.iter());
    assert_eq!(wide_string.to_vec(), wide_expected);
    assert_eq!(wide_string.bits().bit_len(), wide_expected.len() * 7);

    wide_source.clear();
    wide_string.extend(wide_source.iter());
    assert_eq!(wide_string.to_vec(), wide_expected);

    let mut bytes = PackedString::<super::SparseByte, 8>::new();
    let byte_source = [
        super::SparseByte::Zero,
        super::SparseByte::Maximum,
        super::SparseByte::Middle,
    ];
    bytes.extend(byte_source.iter());
    assert_eq!(bytes.to_vec(), byte_source);
    assert_eq!(bytes.bits().bit_len(), 24);
}

#[test]
fn packed_extend_appends_owned_codes_without_changing_the_prefix() {
    let mut oct_string = packed_as::<Oct, 3>(&[7; 20], super::oct);
    let original = oct_string.to_vec();
    oct_string.extend(core::iter::empty::<Oct>());
    assert_eq!(oct_string.to_vec(), original);

    let appended = [Oct::V0, Oct::V7, Oct::V3, Oct::V4, Oct::V1, Oct::V6];
    let mut expected = original.clone();
    expected.extend(appended);
    oct_string.extend(appended);
    assert_eq!(oct_string.to_vec(), expected);
    assert_eq!(oct_string.char_len(), expected.len());
    assert_eq!(oct_string.bits().bit_len(), expected.len() * 3);

    let expected_bits = BitString::from_iter(
        expected
            .iter()
            .flat_map(|character| (0..3).map(move |offset| character.code() & (1 << offset) != 0)),
    );
    assert_eq!(oct_string.bits().words(), expected_bits.words());

    let mut wide_string = PackedString::<WideCode, 7>::new();
    let wide_values = [WideCode(0), WideCode(127), WideCode(64), WideCode(1)];
    wide_string.extend(wide_values);
    wide_string.extend([WideCode(126), WideCode(2), WideCode(127)]);
    assert_eq!(
        wide_string.to_vec(),
        vec![
            WideCode(0),
            WideCode(127),
            WideCode(64),
            WideCode(1),
            WideCode(126),
            WideCode(2),
            WideCode(127),
        ]
    );
    assert_eq!(wide_string.bits().bit_len(), 7 * 7);
}

#[test]
fn packed_views_and_editing_remain_character_aligned() {
    let mut string = PackedString::<super::Symbol, 2>::from_chars([
        super::Symbol::Zero,
        super::Symbol::One,
        super::Symbol::Two,
        super::Symbol::One,
    ]);

    let view = string.as_packed_str();
    assert_eq!(view.char_len(), 4);
    assert_eq!(
        view.slice(UsizeCO::checked_from_start_len(1, 2).unwrap())
            .get(0),
        Some(super::Symbol::One)
    );
    assert_eq!(view.find(view.slice_until(1)), Some(0));
    assert!(view.contains(view.slice(UsizeCO::checked_from_start_len(1, 1).unwrap())));

    let haystack_string = PackedString::<super::Symbol, 2>::from_chars([
        super::Symbol::Zero,
        super::Symbol::Zero,
        super::Symbol::One,
    ]);
    let needle_string = PackedString::<super::Symbol, 2>::from_chars([super::Symbol::Two]);
    let haystack = haystack_string.as_packed_str();
    let needle = needle_string.as_packed_str();
    assert!(!haystack.contains(needle));
    assert_eq!(haystack.find(needle), None);
    assert_eq!(haystack.rfind(needle), None);
    assert!(!haystack.matches_at(1, needle));

    string.insert(2, super::Symbol::Zero);
    assert_eq!(string.remove(2), super::Symbol::Zero);
    assert_eq!(
        string.reverse().to_vec(),
        vec![
            super::Symbol::One,
            super::Symbol::Two,
            super::Symbol::One,
            super::Symbol::Zero
        ]
    );
    string.retain(|symbol| symbol != super::Symbol::Two);
    assert_eq!(
        string.to_vec(),
        vec![super::Symbol::Zero, super::Symbol::One, super::Symbol::One]
    );
}

#[test]
fn packed_string_set_replaces_only_one_character_slot() {
    let mut string = packed(&[0, 1, 2, 1, 0]);
    let original_bits = string.bits().clone();
    assert_eq!(string.set(0, super::Symbol::Two), Some(super::Symbol::Zero));
    assert_eq!(string.get(0), Some(super::Symbol::Two));
    assert_eq!(string.get(1), Some(super::Symbol::One));
    assert_eq!(string.get(4), Some(super::Symbol::Zero));
    assert_eq!(string.char_len(), 5);
    assert_eq!(string.bits().bit_len(), original_bits.bit_len());

    assert_eq!(
        string.set(4, super::Symbol::Zero),
        Some(super::Symbol::Zero)
    );
    assert_eq!(string.set(99, super::Symbol::One), None);
    assert_eq!(string.get(0), Some(super::Symbol::Two));
    assert_eq!(string.get(4), Some(super::Symbol::Zero));

    let mut binary = packed_as::<super::PackedSymbol, 1>(&[0, 1, 0], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary.set(1, super::PackedSymbol::Zero),
        Some(super::PackedSymbol::One)
    );
    assert_eq!(binary.to_vec(), vec![super::PackedSymbol::Zero; 3]);
    assert_eq!(binary.bits().bit_len(), 3);

    let mut bytes = packed_as::<super::SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes.set(1, super::SparseByte::Zero),
        Some(super::SparseByte::Maximum)
    );
    assert_eq!(
        bytes.to_vec(),
        vec![
            super::SparseByte::Zero,
            super::SparseByte::Zero,
            super::SparseByte::Middle,
        ]
    );
    assert_eq!(bytes.bits().bit_len(), 24);

    let oct_codes: Vec<u8> = (0..22).map(|index| index as u8 % 8).collect();
    let mut oct_string = packed_as::<Oct, 3>(&oct_codes, super::oct);
    assert_eq!(oct_string.set(21, Oct::V7), Some(Oct::V5));
    assert_eq!(oct_string.get(20), Some(Oct::V4));
    assert_eq!(oct_string.get(21), Some(Oct::V7));
    assert_eq!(oct_string.bits().bit_len(), 66);

    let wide_codes: Vec<u8> = (0..10).map(|index| (index * 11) as u8 % 128).collect();
    let mut wide_string = packed_as::<WideCode, 7>(&wide_codes, super::wide);
    assert_eq!(wide_string.set(9, WideCode(127)), Some(WideCode(99)));
    assert_eq!(wide_string.get(8), Some(WideCode(88)));
    assert_eq!(wide_string.get(9), Some(WideCode(127)));
    assert_eq!(wide_string.bits().bit_len(), 70);
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
        assert_edits::<Oct, 3>(&initial3, &replacement3, insert_index, remove_index, replace_start, 2, super::oct);
        assert_edits::<WideCode, 7>(&initial7, &replacement7, insert_index, remove_index, replace_start, 2, super::wide);
    }
}
