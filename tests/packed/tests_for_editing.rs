use super::{Oct, PackedSymbol, WideCode, packed, packed_as, symbol};
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
fn packed_pop_returns_final_codes_and_shrinks_to_empty() {
    let mut empty = PackedString::<Oct, 3>::new();
    assert_eq!(empty.pop(), None);
    assert!(empty.bits().words().is_empty());

    let mut maximum =
        PackedString::<super::SparseByte, 8>::from_chars([super::SparseByte::Maximum]);
    assert_eq!(maximum.pop(), Some(super::SparseByte::Maximum));
    assert!(maximum.is_empty());
    assert_eq!(maximum.pop(), None);

    let mut oct_codes: Vec<_> = (0..23)
        .map(|index| if index == 22 { 0 } else { index as u8 % 8 })
        .map(super::oct)
        .collect();
    let mut oct_string = PackedString::from_chars(oct_codes.clone());
    while let Some(expected) = oct_codes.pop() {
        assert_eq!(oct_string.pop(), Some(expected));
        assert_eq!(oct_string.to_vec(), oct_codes);
        assert_eq!(oct_string.char_len(), oct_codes.len());
        assert_eq!(oct_string.bits().bit_len(), oct_codes.len() * 3);
        assert_eq!(
            oct_string.bits().words().len(),
            (oct_codes.len() * 3).div_ceil(64)
        );
        assert_eq!(oct_string.last(), oct_codes.last().copied());
    }
    assert_eq!(oct_string.pop(), None);

    let mut wide_codes: Vec<_> = (0..10)
        .map(|index| {
            if index == 9 {
                127
            } else {
                (index * 11) as u8 % 128
            }
        })
        .map(super::wide)
        .collect();
    let mut wide_string = PackedString::from_chars(wide_codes.clone());
    while let Some(expected) = wide_codes.pop() {
        assert_eq!(wide_string.pop(), Some(expected));
        assert_eq!(wide_string.last(), wide_codes.last().copied());
        assert_eq!(wide_string.bits().bit_len(), wide_codes.len() * 7);
    }
    assert!(wide_string.is_empty());
}

#[test]
fn packed_push_packed_string_concatenates_raw_payloads() {
    let mut oct_left = packed_as::<Oct, 3>(&[7; 21], super::oct);
    let oct_right_codes = [0, 7, 3];
    let oct_right = packed_as::<Oct, 3>(&oct_right_codes, super::oct);
    let oct_right_snapshot = oct_right.clone();
    let mut oct_oracle = oct_left.to_vec();
    oct_oracle.extend(oct_right.to_vec());

    oct_left.push_packed_string(&oct_right);
    assert_eq!(oct_left.to_vec(), oct_oracle);
    assert_eq!(oct_left.char_len(), 24);
    assert_eq!(oct_left.bits().bit_len(), 24 * 3);
    assert!(oct_right == oct_right_snapshot);
    assert_eq!(oct_left.get(21), Some(Oct::V0));
    assert_eq!(oct_left.get(22), Some(Oct::V7));
    assert_eq!(oct_left.get(23), Some(Oct::V3));

    let expected_bits = BitString::from_iter(
        oct_oracle
            .iter()
            .flat_map(|character| (0..3).map(move |offset| character.code() & (1 << offset) != 0)),
    );
    assert_eq!(oct_left.bits().words(), expected_bits.words());

    let empty = PackedString::<Oct, 3>::new();
    let before_empty_append = oct_left.clone();
    oct_left.push_packed_string(&empty);
    assert!(oct_left == before_empty_append);

    let original = packed_as::<Oct, 3>(&[0, 7, 3, 1], super::oct);
    let mut duplicated = original.clone();
    let source = original.clone();
    duplicated.push_packed_string(&source);
    assert_eq!(
        duplicated.to_vec(),
        [0, 7, 3, 1, 0, 7, 3, 1].map(super::oct)
    );

    let mut wide_left = packed_as::<WideCode, 7>(&[126; 9], super::wide);
    let wide_right = packed_as::<WideCode, 7>(&[0, 127, 64], super::wide);
    wide_left.push_packed_string(&wide_right);
    assert_eq!(wide_left.char_len(), 12);
    assert_eq!(wide_left.bits().bit_len(), 12 * 7);
    assert_eq!(wide_left.get(9), Some(WideCode(0)));
    assert_eq!(wide_left.get(10), Some(WideCode(127)));
    assert_eq!(wide_left.get(11), Some(WideCode(64)));

    let mut bytes = PackedString::<super::SparseByte, 8>::new();
    let byte_right =
        PackedString::from_chars([super::SparseByte::Zero, super::SparseByte::Maximum]);
    bytes.push_packed_string(&byte_right);
    assert_eq!(
        bytes.to_vec(),
        vec![super::SparseByte::Zero, super::SparseByte::Maximum]
    );
}

#[test]
fn packed_push_appends_one_aligned_code_at_a_time() {
    let mut oct_string = PackedString::<Oct, 3>::new();
    let mut oracle = Vec::new();
    let values: Vec<_> = (0..22).map(|index| super::oct(index as u8 % 8)).collect();

    for value in values.iter().copied() {
        let old_prefix = oct_string.to_vec();
        let old_bit_len = oct_string.bits().bit_len();
        oct_string.push(value);
        oracle.push(value);
        assert_eq!(oct_string.to_vec(), oracle);
        assert_eq!(oct_string.bits().bit_len(), old_bit_len + 3);
        assert_eq!(oct_string.to_vec()[..old_prefix.len()], old_prefix);
    }
    assert_eq!(oct_string.get(21), Some(Oct::V5));
    assert_eq!(oct_string.char_len(), 22);

    oct_string.push(Oct::V0);
    oracle.push(Oct::V0);
    assert_eq!(oct_string.to_vec(), oracle);
    assert_eq!(oct_string.bits().bit_len(), 23 * 3);
    assert_eq!(oct_string.get(22), Some(Oct::V0));
    assert!(!oct_string.is_empty());

    let mut binary = PackedString::<PackedSymbol, 1>::new();
    binary.push(PackedSymbol::One);
    binary.push(PackedSymbol::Zero);
    assert_eq!(binary.to_vec(), vec![PackedSymbol::One, PackedSymbol::Zero]);
    assert_eq!(binary.bits().bit_len(), 2);

    let mut bytes = PackedString::<super::SparseByte, 8>::new();
    bytes.push(super::SparseByte::Zero);
    bytes.push(super::SparseByte::Maximum);
    assert_eq!(
        bytes.to_vec(),
        vec![super::SparseByte::Zero, super::SparseByte::Maximum]
    );
    assert_eq!(bytes.bits().bit_len(), 16);
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
