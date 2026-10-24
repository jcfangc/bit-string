use super::{
    Oct, PackedString, PackedSymbol, SparseByte, Symbol, WideCode, oct, packed, packed_as, wide,
};
use bit_string::{ContainsOptions, FindOptions, traits::PackedChar};
use int_intervals::UsizeCO;

#[path = "tests_for_matching/tests_for_vec_search_semantics.rs"]
mod tests_for_vec_search_semantics;

#[path = "tests_for_matching/tests_for_prefix_suffix_stripping.rs"]
mod tests_for_prefix_suffix_stripping;

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

    assert_eq!(
        haystack_view.find_with(needle_view, FindOptions::default()),
        expected_find
    );
    assert_eq!(
        haystack_view.rfind_with(needle_view, FindOptions::default()),
        expected_rfind
    );
    assert_eq!(
        haystack_view.contains_with(needle_view, ContainsOptions::default()),
        expected_find.is_some()
    );
    assert_eq!(
        haystack_string.find_with(&needle_string, FindOptions::default()),
        expected_find
    );
    assert_eq!(
        haystack_string.rfind_with(&needle_string, FindOptions::default()),
        expected_rfind
    );
    assert_eq!(
        haystack_string.contains_with(&needle_string, ContainsOptions::default()),
        expected_find.is_some()
    );
}

#[path = "tests_for_matching/tests_for_character_alignment.rs"]
mod tests_for_character_alignment;

#[path = "tests_for_matching/tests_for_search.rs"]
mod tests_for_search;

#[path = "tests_for_matching/tests_for_cross_word_boundaries.rs"]
mod tests_for_cross_word_boundaries;
