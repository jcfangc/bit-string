use alloc::vec;
use alloc::vec::Vec;
use int_intervals::UsizeCO;
use proptest::prelude::*;

use crate::BitString;

use super::{contains, find, rfind};

fn bit_string_from_codes<const BITS: u8>(codes: &[u8]) -> BitString {
    BitString::from_bool_iter(
        codes
            .iter()
            .copied()
            .flat_map(|code| (0..BITS).map(move |bit| code & (1 << bit) != 0)),
    )
}

fn assert_search_crosses_word<const BITS: u8>() {
    let width = usize::from(BITS);
    let candidate_start = (0..64)
        .find(|&start| ((start + 1) * width) % 64 + 3 * width > 64)
        .unwrap();
    let candidate_bit_start = (candidate_start + 1) * width;
    assert!(candidate_bit_start % 64 + 3 * width > 64);

    let mut haystack_codes = vec![1; candidate_start + 1];
    haystack_codes.extend([0, 1, 0]);
    haystack_codes.extend([1, 1, 1]);
    let haystack_source = bit_string_from_codes::<BITS>(&haystack_codes);
    let haystack = haystack_source.as_bit_str().slice_from(width);

    let needle_source = bit_string_from_codes::<BITS>(&[1, 0, 1, 0, 1]);
    let needle_bit_len = 3 * width;
    let needle_interval = UsizeCO::checked_from_start_len(width, needle_bit_len).unwrap();
    let needle = needle_source.as_bit_str().slice(needle_interval);
    assert_eq!(needle.bit_len(), needle_bit_len);

    assert_eq!(
        find::<BITS, false, false>(haystack, needle),
        Some(candidate_start)
    );
    assert_eq!(
        rfind::<BITS, false, false>(haystack, needle),
        Some(candidate_start)
    );
    assert!(contains::<BITS, false, false>(haystack, needle));
}

fn assert_search_matches_reference<const BITS: u8>(haystack_values: &[u8], needle_values: &[u8]) {
    let width = usize::from(BITS);
    let code_mask = ((1_u16 << BITS) - 1) as u8;
    let haystack_codes: Vec<u8> = haystack_values
        .iter()
        .map(|code| code & code_mask)
        .collect();
    let needle_codes: Vec<u8> = needle_values.iter().map(|code| code & code_mask).collect();

    let mut haystack_source_codes = vec![1 & code_mask];
    haystack_source_codes.extend_from_slice(&haystack_codes);
    let haystack_source = bit_string_from_codes::<BITS>(&haystack_source_codes);
    let haystack = haystack_source.as_bit_str().slice_from(width);

    let mut needle_source_codes = vec![1 & code_mask];
    needle_source_codes.extend_from_slice(&needle_codes);
    let needle_source = bit_string_from_codes::<BITS>(&needle_source_codes);
    let needle = needle_source.as_bit_str().slice_from(width);

    let last_start = haystack_codes.len().saturating_sub(needle_codes.len());
    let expected_find = (0..=last_start).find(|&start| {
        haystack_codes.get(start..start + needle_codes.len()) == Some(needle_codes.as_slice())
    });
    let expected_rfind = (0..=last_start).rev().find(|&start| {
        haystack_codes.get(start..start + needle_codes.len()) == Some(needle_codes.as_slice())
    });

    assert_eq!(find::<BITS, false, false>(haystack, needle), expected_find);
    assert_eq!(
        rfind::<BITS, false, false>(haystack, needle),
        expected_rfind
    );
    assert_eq!(
        contains::<BITS, false, false>(haystack, needle),
        expected_find.is_some()
    );
}

#[test]
fn all_widths_find_a_character_aligned_match_crossing_a_word() {
    assert_search_crosses_word::<1>();
    assert_search_crosses_word::<2>();
    assert_search_crosses_word::<3>();
    assert_search_crosses_word::<4>();
    assert_search_crosses_word::<5>();
    assert_search_crosses_word::<6>();
    assert_search_crosses_word::<7>();
    assert_search_crosses_word::<8>();
}

proptest! {
    #[test]
    fn all_widths_match_the_character_slice_oracle(
        haystack_codes in prop::collection::vec(any::<u8>(), 0..=128),
        needle_codes in prop::collection::vec(any::<u8>(), 0..=32),
    ) {
        assert_search_matches_reference::<1>(&haystack_codes, &needle_codes);
        assert_search_matches_reference::<2>(&haystack_codes, &needle_codes);
        assert_search_matches_reference::<3>(&haystack_codes, &needle_codes);
        assert_search_matches_reference::<4>(&haystack_codes, &needle_codes);
        assert_search_matches_reference::<5>(&haystack_codes, &needle_codes);
        assert_search_matches_reference::<6>(&haystack_codes, &needle_codes);
        assert_search_matches_reference::<7>(&haystack_codes, &needle_codes);
        assert_search_matches_reference::<8>(&haystack_codes, &needle_codes);
    }
}
