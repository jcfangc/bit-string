use bit_string::{BitString, PackedString, packed, traits::PackedChar};
use int_intervals::UsizeCO;
use proptest::prelude::*;

#[packed(bits = 1)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackedSymbol {
    Zero = 0,
    One = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[packed(bits = 2)]
enum Symbol {
    Zero = 0,
    One = 1,
    Two = 2,
}

#[packed(bits = 3)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Oct {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WideCode(u8);

impl PackedChar<7> for WideCode {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code < 128).then_some(Self(code))
    }
}

fn symbol(code: u8) -> Symbol {
    match code {
        0 => Symbol::Zero,
        1 => Symbol::One,
        2 => Symbol::Two,
        _ => unreachable!(),
    }
}

fn oct(code: u8) -> Oct {
    match code {
        0 => Oct::V0,
        1 => Oct::V1,
        2 => Oct::V2,
        3 => Oct::V3,
        4 => Oct::V4,
        5 => Oct::V5,
        6 => Oct::V6,
        7 => Oct::V7,
        _ => unreachable!(),
    }
}

fn wide(code: u8) -> WideCode {
    WideCode(code)
}

fn packed(codes: &[u8]) -> PackedString<Symbol, 2> {
    PackedString::from_chars(codes.iter().copied().map(symbol))
}

fn packed_as<C, const BITS: u8>(codes: &[u8], decode: fn(u8) -> C) -> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    PackedString::from_chars(codes.iter().copied().map(decode))
}

fn matches_at(haystack: &[u8], start: usize, needle: &[u8]) -> bool {
    haystack
        .get(start..start.saturating_add(needle.len()))
        .is_some_and(|window| window == needle)
}

fn assert_access_slice_order<C, const BITS: u8>(
    left: &[u8],
    right: &[u8],
    start: usize,
    len: usize,
    decode: fn(u8) -> C,
) where
    C: PackedChar<BITS> + core::fmt::Debug,
{
    let string = packed_as(left, decode);
    assert_eq!(string.char_len(), left.len());
    let view = string.as_packed_str();
    for index in 0..=left.len() + 1 {
        assert_eq!(string.get(index), left.get(index).copied().map(decode));
        assert_eq!(view.get(index), left.get(index).copied().map(decode));
    }

    let oracle_start = start.min(left.len());
    let oracle_end = start.saturating_add(len).min(left.len()).max(oracle_start);
    let slice = string.slice(UsizeCO::checked_from_start_len(start, len).unwrap());
    assert_eq!(
        slice.to_vec(),
        left[oracle_start..oracle_end]
            .iter()
            .copied()
            .map(decode)
            .collect::<Vec<_>>()
    );

    let right_string = packed_as(right, decode);
    assert_eq!(string.cmp(&right_string), left.cmp(&right));
    assert_eq!(
        string.as_packed_str().cmp(&right_string.as_packed_str()),
        left.cmp(&right)
    );
}

fn assert_matching<C, const BITS: u8>(haystack: &[u8], needle: &[u8], decode: fn(u8) -> C)
where
    C: PackedChar<BITS>,
{
    let haystack_string = packed_as(haystack, decode);
    let needle_string = packed_as(needle, decode);
    let haystack_view = haystack_string.as_packed_str();
    let needle_view = needle_string.as_packed_str();
    let max_start = haystack.len().saturating_sub(needle.len());
    let expected_find = (0..=max_start).find(|&start| matches_at(haystack, start, needle));
    let expected_rfind = (0..=max_start)
        .rev()
        .find(|&start| matches_at(haystack, start, needle));

    assert_eq!(haystack_view.find(needle_view), expected_find);
    assert_eq!(haystack_view.rfind(needle_view), expected_rfind);
    assert_eq!(haystack_view.contains(needle_view), expected_find.is_some());
}

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

proptest! {
    #[test]
    fn packed_access_slice_and_order_match_vec_oracles(
        left in prop::collection::vec(0u8..=2, 0..=32),
        right in prop::collection::vec(0u8..=2, 0..=32),
        start in 0usize..40,
        len in 1usize..40,
    ) {
        let string = packed(&left);
        prop_assert_eq!(string.char_len(), left.len());
        let view = string.as_packed_str();
        for index in 0..=left.len() + 1 {
            prop_assert_eq!(string.get(index), left.get(index).copied().map(symbol));
            prop_assert_eq!(view.get(index), left.get(index).copied().map(symbol));
        }

        let oracle_start = start.min(left.len());
        let oracle_end = start.saturating_add(len).min(left.len()).max(oracle_start);
        let slice = string.slice(UsizeCO::checked_from_start_len(start, len).unwrap());
        prop_assert_eq!(slice.to_vec(), left[oracle_start..oracle_end].iter().copied().map(symbol).collect::<Vec<_>>());

        let right_string = packed(&right);
        prop_assert_eq!(string.cmp(&right_string), left.cmp(&right));
        prop_assert_eq!(string.as_packed_str().cmp(&right_string.as_packed_str()), left.cmp(&right));
    }

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

proptest! {
    #[test]
    fn three_and_seven_bit_access_slice_and_order_cross_word_boundaries(
        left3 in prop::collection::vec(0u8..=7, 22..=128),
        right3 in prop::collection::vec(0u8..=7, 22..=128),
        left7 in prop::collection::vec(0u8..=127, 10..=128),
        right7 in prop::collection::vec(0u8..=127, 10..=128),
        start in 0usize..160,
        len in 1usize..80,
    ) {
        assert_access_slice_order::<Oct, 3>(&left3, &right3, start, len, oct);
        assert_access_slice_order::<WideCode, 7>(&left7, &right7, start, len, wide);
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
        assert_edits::<Oct, 3>(
            &initial3,
            &replacement3,
            insert_index,
            remove_index,
            replace_start,
            2,
            oct,
        );
        assert_edits::<WideCode, 7>(
            &initial7,
            &replacement7,
            insert_index,
            remove_index,
            replace_start,
            2,
            wide,
        );
    }
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
fn packed_attribute_encoding_round_trips_through_packed_string() {
    let string = PackedString::<Symbol, 2>::from_chars([Symbol::Two, Symbol::Zero, Symbol::One]);
    assert_eq!(string.get(0), Some(Symbol::Two));
    assert_eq!(string.get(1), Some(Symbol::Zero));
    assert_eq!(string.get(2), Some(Symbol::One));
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ManualSymbol;

impl PackedChar<1> for ManualSymbol {
    fn code(self) -> u8 {
        0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code == 0).then_some(Self)
    }
}

#[test]
fn manual_packed_char_implementations_remain_supported() {
    assert_eq!(
        PackedString::<ManualSymbol, 1>::from_chars([ManualSymbol]).char_len(),
        1
    );
}

#[test]
fn attribute_macro_generates_packed_char_impl() {
    let string =
        PackedString::<PackedSymbol, 1>::from_chars([PackedSymbol::One, PackedSymbol::Zero]);
    assert_eq!(string.get(0), Some(PackedSymbol::One));
}

#[test]
fn packed_views_and_editing_remain_character_aligned() {
    let mut string = PackedString::<Symbol, 2>::from_chars([
        Symbol::Zero,
        Symbol::One,
        Symbol::Two,
        Symbol::One,
    ]);

    let view = string.as_packed_str();
    assert_eq!(view.char_len(), 4);
    assert_eq!(
        view.slice(UsizeCO::checked_from_start_len(1, 2).unwrap())
            .get(0),
        Some(Symbol::One)
    );
    assert_eq!(view.find(view.slice_until(1)), Some(0));
    assert!(view.contains(view.slice(UsizeCO::checked_from_start_len(1, 1).unwrap())));

    let haystack_string =
        PackedString::<Symbol, 2>::from_chars([Symbol::Zero, Symbol::Zero, Symbol::One]);
    let needle_string = PackedString::<Symbol, 2>::from_chars([Symbol::Two]);
    let haystack = haystack_string.as_packed_str();
    let needle = needle_string.as_packed_str();
    assert!(!haystack.contains(needle));
    assert_eq!(haystack.find(needle), None);
    assert_eq!(haystack.rfind(needle), None);
    assert!(!haystack.matches_at(1, needle));

    string.insert(2, Symbol::Zero);
    assert_eq!(string.remove(2), Symbol::Zero);
    assert_eq!(
        string.reverse().to_vec(),
        vec![Symbol::One, Symbol::Two, Symbol::One, Symbol::Zero]
    );
    string.retain(|symbol| symbol != Symbol::Two);
    assert_eq!(
        string.to_vec(),
        vec![Symbol::Zero, Symbol::One, Symbol::One]
    );
}

#[test]
fn from_bits_rejects_misaligned_and_unknown_codes() {
    assert!(PackedString::<Symbol, 2>::from_bits(BitString::from_iter([true])).is_none());
    assert!(PackedString::<Symbol, 2>::from_bits(BitString::from_iter([true, true])).is_none());
}

#[test]
fn ordering_uses_packed_code_values() {
    let one = PackedString::<Symbol, 2>::from_chars([Symbol::One]);
    let two = PackedString::<Symbol, 2>::from_chars([Symbol::Two]);
    assert!(one < two);
    assert!(one.as_packed_str() < two.as_packed_str());
}
