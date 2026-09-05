use super::{Oct, PackedString, WideCode, oct, packed, packed_as, wide};
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
fn packed_string_matches_at_is_character_aligned_and_bounds_checked() {
    let haystack = packed_as::<Oct, 3>(&[0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3], oct);
    let needle = packed_as::<Oct, 3>(&[4, 5], oct);
    assert!(haystack.matches_at(4, &needle));
    assert!(!haystack.matches_at(3, &needle));
    assert!(!haystack.matches_at(11, &needle));
    assert!(!haystack.matches_at(usize::MAX, &needle));

    let empty = PackedString::<Oct, 3>::new();
    assert!(haystack.matches_at(haystack.char_len(), &empty));
    assert!(!haystack.matches_at(haystack.char_len() + 1, &empty));

    let wide_haystack = packed_as::<WideCode, 7>(&[0, 11, 22, 33, 44, 55, 66, 77, 88, 99], wide);
    let wide_needle = packed_as::<WideCode, 7>(&[88, 99], wide);
    assert!(wide_haystack.matches_at(8, &wide_needle));
    assert!(!wide_haystack.matches_at(7, &wide_needle));
    assert!(!wide_haystack.matches_at(9, &wide_needle));
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

#[test]
fn packed_string_starts_with_compares_character_prefixes() {
    let receiver = packed(&[0, 1, 2, 1, 0]);
    assert!(receiver.starts_with(&packed(&[0, 1, 2, 1, 0])));
    assert!(receiver.starts_with(&packed(&[0, 1, 2])));
    assert!(!receiver.starts_with(&packed(&[0, 2])));
    assert!(!receiver.starts_with(&packed(&[0, 1, 2, 1, 0, 1])));
    assert!(receiver.starts_with(&PackedString::<super::Symbol, 2>::new()));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_receiver = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_prefix = packed_as::<Oct, 3>(&oct_codes[..22], oct);
    assert!(oct_receiver.starts_with(&oct_prefix));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_receiver = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_prefix = packed_as::<WideCode, 7>(&wide_codes[..10], wide);
    assert!(wide_receiver.starts_with(&wide_prefix));
}

#[test]
fn packed_str_ends_with_compares_character_aligned_suffixes() {
    let owner = packed(&[0, 1, 2, 1, 0]);
    let receiver = owner.as_packed_str();
    let exact = packed(&[0, 1, 2, 1, 0]);
    let shorter = packed(&[2, 1, 0]);
    let different = packed(&[1, 1, 0]);
    let oversized = packed(&[0, 1, 2, 1, 0, 1]);
    assert!(receiver.ends_with(exact.as_packed_str()));
    assert!(receiver.ends_with(shorter.as_packed_str()));
    assert!(!receiver.ends_with(different.as_packed_str()));
    assert!(!receiver.ends_with(oversized.as_packed_str()));

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    let nonempty_owner = packed(&[0]);
    assert!(empty.ends_with(empty));
    assert!(receiver.ends_with(empty));
    assert!(!empty.ends_with(nonempty_owner.as_packed_str()));

    let offset_receiver = receiver.slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_suffix = receiver.slice(UsizeCO::checked_from_start_len(2, 2).unwrap());
    let offset_different = receiver.slice(UsizeCO::checked_from_start_len(1, 2).unwrap());
    assert!(offset_receiver.ends_with(offset_suffix));
    assert!(!offset_receiver.ends_with(offset_different));

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert!(
        binary
            .as_packed_str()
            .ends_with(binary.as_packed_str().slice_from(1))
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[0, 255], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert!(
        bytes
            .as_packed_str()
            .ends_with(bytes.as_packed_str().slice_from(1))
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_receiver = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert!(oct_receiver.ends_with(oct_receiver.slice_from(2)));
    assert!(!oct_receiver.ends_with(oct_receiver.slice_from(1).slice_until(2)));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_receiver = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert!(wide_receiver.ends_with(wide_receiver.slice_from(2)));
    assert!(!wide_receiver.ends_with(wide_receiver.slice_from(1).slice_until(2)));
}

#[test]
fn packed_string_ends_with_compares_character_suffixes() {
    let receiver = packed(&[0, 1, 2, 1, 0]);
    assert!(receiver.ends_with(&packed(&[0, 1, 2, 1, 0])));
    assert!(receiver.ends_with(&packed(&[2, 1, 0])));
    assert!(!receiver.ends_with(&packed(&[1, 1, 0])));
    assert!(!receiver.ends_with(&packed(&[0, 1, 2, 1, 0, 1])));
    assert!(receiver.ends_with(&PackedString::<super::Symbol, 2>::new()));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_receiver = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_suffix = packed_as::<Oct, 3>(&oct_codes[2..], oct);
    assert!(oct_receiver.ends_with(&oct_suffix));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_receiver = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_suffix = packed_as::<WideCode, 7>(&wide_codes[6..], wide);
    assert!(wide_receiver.ends_with(&wide_suffix));
}

#[test]
fn packed_str_contains_searches_character_aligned_windows() {
    let haystack_owner = packed(&[0, 0, 0]);
    let haystack = haystack_owner.as_packed_str();
    let repeated = packed(&[0, 0]);
    let final_haystack_owner = packed(&[1, 2, 0]);
    let final_haystack = final_haystack_owner.as_packed_str();
    let final_window = packed(&[2, 0]);
    let different = packed(&[1, 2]);
    let oversized = packed(&[0, 0, 0, 1]);

    assert!(haystack.contains(repeated.as_packed_str()));
    assert!(final_haystack.contains(final_window.as_packed_str()));
    assert!(!haystack.contains(different.as_packed_str()));
    assert!(!haystack.contains(oversized.as_packed_str()));
    assert!(haystack.contains(haystack.slice_until(0)));

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    let singleton_owner = packed(&[0]);
    assert!(empty.contains(empty));
    assert!(!empty.contains(singleton_owner.as_packed_str()));

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_haystack = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = offset_haystack.slice_from(1);
    let offset_different_owner = packed(&[2, 2]);
    let offset_different = offset_different_owner.as_packed_str();
    assert!(offset_haystack.contains(offset_needle));
    assert!(!offset_haystack.contains(offset_different));

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert!(
        binary
            .as_packed_str()
            .contains(binary.as_packed_str().slice_from(1))
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[0, 255], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert!(
        bytes
            .as_packed_str()
            .contains(bytes.as_packed_str().slice_from(1))
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_different_owner = packed_as::<Oct, 3>(&[5, 4], oct);
    assert!(oct_haystack.contains(oct_haystack.slice_from(1)));
    assert!(!oct_haystack.contains(oct_different_owner.as_packed_str()));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    let wide_different_owner = packed_as::<WideCode, 7>(&[11, 0], wide);
    assert!(wide_haystack.contains(wide_haystack.slice_from(1)));
    assert!(!wide_haystack.contains(wide_different_owner.as_packed_str()));
}

#[test]
fn packed_string_contains_searches_character_aligned_windows() {
    let haystack = packed(&[0, 1, 0, 1, 2, 1]);
    assert!(haystack.contains(&packed(&[0, 1, 2])));
    assert!(haystack.contains(&packed(&[1, 2, 1])));
    assert!(!haystack.contains(&packed(&[2, 0])));
    assert!(!haystack.contains(&packed(&[0, 1, 0, 1, 2, 1, 0])));
    assert!(haystack.contains(&PackedString::<super::Symbol, 2>::new()));

    let empty = PackedString::<super::Symbol, 2>::new();
    assert!(!empty.contains(&packed(&[0])));

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_haystack = packed_as::<Oct, 3>(&oct_codes, oct);
    assert!(oct_haystack.contains(&packed_as::<Oct, 3>(&[4, 5, 6, 7], oct)));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_haystack = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert!(wide_haystack.contains(&packed_as::<WideCode, 7>(&[88, 99], wide)));
}

#[test]
fn packed_string_find_returns_the_earliest_character_index() {
    let haystack = packed(&[0, 1, 0, 1, 2, 1]);
    assert_eq!(haystack.find(&packed(&[0, 1])), Some(0));
    assert_eq!(haystack.find(&packed(&[1, 2])), Some(3));
    assert_eq!(haystack.find(&packed(&[2, 0])), None);
    assert_eq!(haystack.find(&packed(&[0, 1, 0, 1, 2, 1, 0])), None);
    assert_eq!(
        haystack.find(&PackedString::<super::Symbol, 2>::new()),
        Some(0)
    );

    let empty = PackedString::<super::Symbol, 2>::new();
    assert_eq!(empty.find(&empty), Some(0));
    assert_eq!(empty.find(&packed(&[0])), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_haystack = packed_as::<Oct, 3>(&oct_codes, oct);
    assert_eq!(
        oct_haystack.find(&packed_as::<Oct, 3>(&[4, 5, 6], oct)),
        Some(4)
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_haystack = packed_as::<WideCode, 7>(&wide_codes, wide);
    assert_eq!(
        wide_haystack.find(&packed_as::<WideCode, 7>(&[88, 99], wide)),
        Some(8)
    );
}

#[test]
fn packed_str_find_returns_the_earliest_character_index() {
    let repeated_owner = packed(&[0, 0, 0]);
    let repeated_needle = packed(&[0, 0]);
    assert_eq!(
        repeated_owner
            .as_packed_str()
            .find(repeated_needle.as_packed_str()),
        Some(0)
    );

    let final_owner = packed(&[1, 2, 0]);
    let final_needle = packed(&[2, 0]);
    assert_eq!(
        final_owner
            .as_packed_str()
            .find(final_needle.as_packed_str()),
        Some(1)
    );

    let no_match = packed(&[1, 2]);
    let no_match_needle = packed(&[0]);
    let oversized = packed(&[0, 1, 2]);
    assert_eq!(
        no_match
            .as_packed_str()
            .find(no_match_needle.as_packed_str()),
        None
    );
    assert_eq!(
        no_match.as_packed_str().find(oversized.as_packed_str()),
        None
    );

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    assert_eq!(empty.find(empty), Some(0));
    assert_eq!(final_owner.as_packed_str().find(empty), Some(0));

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_haystack = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = packed(&[1, 2]);
    assert_eq!(offset_haystack.find(offset_needle.as_packed_str()), Some(0));

    let binary = packed_as::<super::PackedSymbol, 1>(&[1, 0, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_needle = packed_as::<super::PackedSymbol, 1>(&[1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary.as_packed_str().find(binary_needle.as_packed_str()),
        Some(0)
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_needle = packed_as::<super::SparseByte, 8>(&[255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes.as_packed_str().find(byte_needle.as_packed_str()),
        Some(1)
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert_eq!(oct_haystack.find(oct_haystack.slice_from(1)), Some(1));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert_eq!(wide_haystack.find(wide_haystack.slice_from(1)), Some(1));

    assert_eq!(offset_haystack.char_len(), 3);
    assert_eq!(offset_haystack.get(0), Some(super::Symbol::One));
    assert_eq!(offset_haystack.get(2), Some(super::Symbol::One));
}

#[test]
fn packed_str_rfind_returns_the_latest_character_index() {
    let repeated_owner = packed(&[0, 0, 0]);
    let repeated_needle = packed(&[0, 0]);
    assert_eq!(
        repeated_owner
            .as_packed_str()
            .rfind(repeated_needle.as_packed_str()),
        Some(1)
    );

    let final_owner = packed(&[1, 2, 0]);
    let final_needle = packed(&[2, 0]);
    assert_eq!(
        final_owner
            .as_packed_str()
            .rfind(final_needle.as_packed_str()),
        Some(1)
    );

    let no_match = packed(&[1, 2]);
    let no_match_needle = packed(&[0]);
    let oversized = packed(&[0, 1, 2]);
    assert_eq!(
        no_match
            .as_packed_str()
            .rfind(no_match_needle.as_packed_str()),
        None
    );
    assert_eq!(
        no_match.as_packed_str().rfind(oversized.as_packed_str()),
        None
    );

    let empty_owner = packed(&[]);
    let empty = empty_owner.as_packed_str();
    assert_eq!(empty.rfind(empty), Some(0));
    assert_eq!(final_owner.as_packed_str().rfind(empty), Some(3));

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_haystack = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_needle = packed(&[1]);
    assert_eq!(
        offset_haystack.rfind(offset_needle.as_packed_str()),
        Some(2)
    );

    let binary = packed_as::<super::PackedSymbol, 1>(&[1, 0, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    let binary_needle = packed_as::<super::PackedSymbol, 1>(&[1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary.as_packed_str().rfind(binary_needle.as_packed_str()),
        Some(2)
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[0, 255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    let byte_needle = packed_as::<super::SparseByte, 8>(&[255, 3], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes.as_packed_str().rfind(byte_needle.as_packed_str()),
        Some(1)
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_haystack = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    assert_eq!(oct_haystack.rfind(oct_haystack.slice_from(1)), Some(1));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_haystack = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    assert_eq!(wide_haystack.rfind(wide_haystack.slice_from(1)), Some(1));

    assert_eq!(offset_haystack.char_len(), 3);
    assert_eq!(offset_haystack.get(0), Some(super::Symbol::One));
    assert_eq!(offset_haystack.get(2), Some(super::Symbol::One));
}

#[test]
fn packed_str_strip_prefix_returns_the_remaining_character_view() {
    let owner = packed(&[0, 1, 2, 1, 0]);
    let receiver = owner.as_packed_str();
    let prefix = packed(&[0, 1]);
    let stripped = receiver.strip_prefix(prefix.as_packed_str()).unwrap();
    assert_eq!(
        stripped.iter().collect::<Vec<_>>(),
        vec![super::Symbol::Two, super::Symbol::One, super::Symbol::Zero]
    );
    assert_eq!(stripped.char_len(), 3);
    assert_eq!(
        stripped.to_packed_string().to_vec(),
        vec![super::Symbol::Two, super::Symbol::One, super::Symbol::Zero]
    );

    let empty_prefix_owner = packed(&[]);
    let empty_prefix = empty_prefix_owner.as_packed_str();
    assert!(receiver.strip_prefix(empty_prefix).unwrap() == receiver);
    assert!(receiver.strip_prefix(receiver).unwrap().is_empty());
    assert!(empty_prefix.strip_prefix(empty_prefix).unwrap().is_empty());

    let different_first = packed(&[1, 1]);
    let different_middle = packed(&[0, 2]);
    let different_last = packed(&[0, 0]);
    let oversized = packed(&[0, 1, 2, 1, 0, 2]);
    assert!(
        receiver
            .strip_prefix(different_first.as_packed_str())
            .is_none()
    );
    assert!(
        receiver
            .strip_prefix(different_middle.as_packed_str())
            .is_none()
    );
    assert!(
        receiver
            .strip_prefix(different_last.as_packed_str())
            .is_none()
    );
    assert!(receiver.strip_prefix(oversized.as_packed_str()).is_none());
    assert!(
        empty_prefix
            .strip_prefix(packed(&[0]).as_packed_str())
            .is_none()
    );

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_receiver = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_prefix = packed(&[1, 2]);
    let offset_stripped = offset_receiver
        .strip_prefix(offset_prefix.as_packed_str())
        .unwrap();
    assert_eq!(
        offset_stripped.iter().collect::<Vec<_>>(),
        vec![super::Symbol::One]
    );
    assert!(
        offset_receiver
            .strip_prefix(offset_receiver.slice_from(offset_receiver.char_len()))
            .unwrap()
            == offset_receiver
    );

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary
            .as_packed_str()
            .strip_prefix(binary.as_packed_str().slice_until(1))
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![super::PackedSymbol::One, super::PackedSymbol::One]
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[255, 3, 0], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes
            .as_packed_str()
            .strip_prefix(bytes.as_packed_str().slice_until(1))
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![super::SparseByte::Middle, super::SparseByte::Zero]
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_receiver = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_stripped = oct_receiver
        .strip_prefix(oct_receiver.slice_until(2))
        .unwrap();
    assert_eq!(
        oct_stripped
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![6, 7]
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_receiver = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    let wide_stripped = wide_receiver
        .strip_prefix(wide_receiver.slice_until(2))
        .unwrap();
    assert_eq!(
        wide_stripped
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![110, 121]
    );

    assert_eq!(offset_receiver.char_len(), 3);
    assert_eq!(offset_receiver.get(0), Some(super::Symbol::One));
    assert_eq!(offset_receiver.get(2), Some(super::Symbol::One));
}

#[test]
fn packed_str_strip_suffix_returns_the_remaining_character_view() {
    let owner = packed(&[0, 1, 2, 1, 0]);
    let receiver = owner.as_packed_str();
    let suffix = packed(&[1, 0]);
    let stripped = receiver.strip_suffix(suffix.as_packed_str()).unwrap();
    assert_eq!(
        stripped.iter().collect::<Vec<_>>(),
        vec![super::Symbol::Zero, super::Symbol::One, super::Symbol::Two]
    );
    assert_eq!(stripped.char_len(), 3);
    assert_eq!(
        stripped.to_packed_string().to_vec(),
        vec![super::Symbol::Zero, super::Symbol::One, super::Symbol::Two]
    );

    let empty_suffix_owner = packed(&[]);
    let empty_suffix = empty_suffix_owner.as_packed_str();
    assert!(receiver.strip_suffix(empty_suffix).unwrap() == receiver);
    assert!(receiver.strip_suffix(receiver).unwrap().is_empty());
    assert!(empty_suffix.strip_suffix(empty_suffix).unwrap().is_empty());

    let different_first = packed(&[0, 2]);
    let different_middle = packed(&[2, 0]);
    let different_last = packed(&[1, 1]);
    let oversized = packed(&[2, 1, 0, 1, 0, 2]);
    assert!(
        receiver
            .strip_suffix(different_first.as_packed_str())
            .is_none()
    );
    assert!(
        receiver
            .strip_suffix(different_middle.as_packed_str())
            .is_none()
    );
    assert!(
        receiver
            .strip_suffix(different_last.as_packed_str())
            .is_none()
    );
    assert!(receiver.strip_suffix(oversized.as_packed_str()).is_none());
    assert!(
        empty_suffix
            .strip_suffix(packed(&[0]).as_packed_str())
            .is_none()
    );

    let offset_owner = packed(&[0, 1, 2, 1, 0]);
    let offset_receiver = offset_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(1, 3).unwrap());
    let offset_suffix = packed(&[2, 1]);
    let offset_stripped = offset_receiver
        .strip_suffix(offset_suffix.as_packed_str())
        .unwrap();
    assert_eq!(
        offset_stripped.iter().collect::<Vec<_>>(),
        vec![super::Symbol::One]
    );
    assert!(
        offset_receiver
            .strip_suffix(offset_receiver.slice_until(0))
            .unwrap()
            == offset_receiver
    );

    let binary = packed_as::<super::PackedSymbol, 1>(&[0, 1, 1], |code| match code {
        0 => super::PackedSymbol::Zero,
        1 => super::PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(
        binary
            .as_packed_str()
            .strip_suffix(binary.as_packed_str().slice_from(1))
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![super::PackedSymbol::Zero]
    );

    let bytes = packed_as::<super::SparseByte, 8>(&[255, 3, 0], |code| match code {
        0 => super::SparseByte::Zero,
        255 => super::SparseByte::Maximum,
        3 => super::SparseByte::Middle,
        _ => unreachable!(),
    });
    assert_eq!(
        bytes
            .as_packed_str()
            .strip_suffix(bytes.as_packed_str().slice_from(1))
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        vec![super::SparseByte::Maximum]
    );

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_receiver = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_stripped = oct_receiver
        .strip_suffix(oct_receiver.slice_from(2))
        .unwrap();
    assert_eq!(
        oct_stripped
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![4, 5]
    );

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_receiver = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 4).unwrap());
    let wide_stripped = wide_receiver
        .strip_suffix(wide_receiver.slice_from(2))
        .unwrap();
    assert_eq!(
        wide_stripped
            .iter()
            .map(|character| character.code())
            .collect::<Vec<_>>(),
        vec![88, 99]
    );

    assert_eq!(offset_receiver.char_len(), 3);
    assert_eq!(offset_receiver.get(0), Some(super::Symbol::One));
    assert_eq!(offset_receiver.get(2), Some(super::Symbol::One));
}
