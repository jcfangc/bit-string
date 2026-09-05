use super::{Oct, WideCode, oct, packed, packed_as, wide};
use bit_string::traits::PackedChar;
use int_intervals::UsizeCO;
use proptest::prelude::*;

fn matches_at(haystack: &[u8], start: usize, needle: &[u8]) -> bool {
    haystack
        .get(start..start.saturating_add(needle.len()))
        .is_some_and(|window| window == needle)
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

#[test]
fn packed_views_match_only_at_character_boundaries() {
    let string = packed(&[0, 1, 2, 1]);
    let view = string.as_packed_str();
    assert_eq!(
        view.slice(UsizeCO::checked_from_start_len(1, 2).unwrap())
            .get(0),
        Some(super::Symbol::One)
    );
    assert_eq!(view.find(view.slice_until(1)), Some(0));
    assert!(view.contains(view.slice(UsizeCO::checked_from_start_len(1, 1).unwrap())));

    let haystack_string = packed(&[0, 0, 1]);
    let needle_string = packed(&[2]);
    let haystack = haystack_string.as_packed_str();
    let needle = needle_string.as_packed_str();
    assert!(!haystack.contains(needle));
    assert_eq!(haystack.find(needle), None);
    assert_eq!(haystack.rfind(needle), None);
    assert!(!haystack.matches_at(1, needle));
}

#[test]
fn packed_str_matches_at_is_character_aligned_and_bounds_checked() {
    let haystack_owner = packed(&[0, 1, 2, 1, 0]);
    let haystack = haystack_owner.as_packed_str();
    let needle_owner = packed(&[1, 2]);
    let needle = needle_owner.as_packed_str();

    assert!(haystack.matches_at(1, needle));
    assert!(!haystack.matches_at(0, needle));
    assert!(!haystack.matches_at(4, needle));
    assert!(!haystack.matches_at(5, needle));
    assert!(!haystack.matches_at(usize::MAX, needle));

    let empty = haystack.slice_until(0);
    assert!(haystack.matches_at(0, empty));
    assert!(haystack.matches_at(haystack.char_len(), empty));
    assert!(!haystack.matches_at(haystack.char_len() + 1, empty));

    let offset_haystack = haystack.slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = haystack.slice(UsizeCO::checked_from_start_len(2, 2).unwrap());
    assert!(offset_haystack.matches_at(1, offset_needle));
    assert!(!offset_haystack.matches_at(0, offset_needle));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_needle = oct_haystack.slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert!(oct_haystack.matches_at(1, oct_needle));
    assert!(!oct_haystack.matches_at(2, oct_needle));
    assert!(!oct_haystack.matches_at(usize::MAX, oct_needle));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    let wide_needle = wide_haystack.slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert!(wide_haystack.matches_at(1, wide_needle));
    assert!(!wide_haystack.matches_at(3, wide_needle));
    assert!(!wide_haystack.matches_at(usize::MAX, wide_needle));
}

#[test]
fn packed_str_starts_with_compares_character_aligned_prefixes() {
    let owner = packed(&[0, 1, 2, 1, 0]);
    let receiver = owner.as_packed_str();
    let exact = packed(&[0, 1, 2, 1, 0]);
    let shorter = packed(&[0, 1, 2]);
    let different = packed(&[0, 2]);
    let oversized = packed(&[0, 1, 2, 1, 0, 1]);
    assert!(receiver.starts_with(exact.as_packed_str()));
    assert!(receiver.starts_with(shorter.as_packed_str()));
    assert!(!receiver.starts_with(different.as_packed_str()));
    assert!(!receiver.starts_with(oversized.as_packed_str()));

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    let nonempty_owner = packed(&[0]);
    assert!(empty.starts_with(empty));
    assert!(receiver.starts_with(empty));
    assert!(!empty.starts_with(nonempty_owner.as_packed_str()));

    let offset_receiver = receiver.slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_prefix = receiver.slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    let offset_different = receiver.slice(UsizeCO::checked_from_start_len(2, 2).unwrap());
    assert!(offset_receiver.starts_with(offset_prefix));
    assert!(!offset_receiver.starts_with(offset_different));

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert!(
        binary
            .as_packed_str()
            .starts_with(binary.as_packed_str().slice_until(1))
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[255, 0], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert!(
        bytes
            .as_packed_str()
            .starts_with(bytes.as_packed_str().slice_until(1))
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_receiver = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert!(oct_receiver.starts_with(oct_receiver.slice_until(2)));
    assert!(!oct_receiver.starts_with(oct_receiver.slice_from(1).slice_until(2)));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_receiver = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert!(wide_receiver.starts_with(wide_receiver.slice_until(2)));
    assert!(!wide_receiver.starts_with(wide_receiver.slice_from(1).slice_until(2)));
}
