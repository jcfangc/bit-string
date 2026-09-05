use bit_string::{BitString, PackedString, packed, traits::PackedChar};
use int_interval::UsizeCO;
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

fn symbol(code: u8) -> Symbol {
    match code {
        0 => Symbol::Zero,
        1 => Symbol::One,
        2 => Symbol::Two,
        _ => unreachable!(),
    }
}

fn packed(codes: &[u8]) -> PackedString<Symbol, 2> {
    PackedString::from_chars(codes.iter().copied().map(symbol))
}

fn matches_at(haystack: &[u8], start: usize, needle: &[u8]) -> bool {
    haystack
        .get(start..start.saturating_add(needle.len()))
        .is_some_and(|window| window == needle)
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
