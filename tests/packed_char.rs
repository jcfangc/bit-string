use bit_string::{BitStr, BitString, PackedStr, PackedString, packed, traits::PackedChar};
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

#[packed(bits = 8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SparseByte {
    Maximum = 255,
    Zero = 0,
    Middle = 3,
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

#[test]
fn generated_sparse_code_round_trip_is_executable() {
    assert_eq!(SparseByte::Zero.code(), 0);
    assert_eq!(SparseByte::Middle.code(), 3);
    assert_eq!(SparseByte::Maximum.code(), u8::MAX);

    for code in 0..=u8::MAX {
        let expected = match code {
            0 => Some(SparseByte::Zero),
            3 => Some(SparseByte::Middle),
            u8::MAX => Some(SparseByte::Maximum),
            _ => None,
        };
        assert_eq!(SparseByte::from_code(code), expected);
    }
}

#[test]
fn packed_str_clone_preserves_sliced_cross_word_views() {
    let oct_codes: Vec<u8> = (0..64).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 24).unwrap());
    let oct_copy = oct_view;
    let oct_clone = oct_view.clone();
    assert!(oct_clone == oct_view);
    assert!(oct_clone == oct_copy);
    assert_eq!(
        oct_clone.iter().collect::<Vec<_>>(),
        oct_codes[20..44]
            .iter()
            .copied()
            .map(oct)
            .collect::<Vec<_>>()
    );
    assert_eq!(oct_clone.iter().count(), 24);

    let wide_codes: Vec<u8> = (0..20).map(|index| (index * 7) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    let wide_clone = wide_view.clone();
    assert!(wide_clone == wide_view);
    assert_eq!(
        wide_clone.iter().collect::<Vec<_>>(),
        wide_codes[8..13]
            .iter()
            .copied()
            .map(wide)
            .collect::<Vec<_>>()
    );
    assert_eq!(wide_clone.first(), Some(wide(wide_codes[8])));
    assert_eq!(wide_clone.last(), Some(wide(wide_codes[12])));

    let empty = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(999, 1).unwrap());
    assert!(empty.clone().is_empty());
    assert!(empty.clone() == empty);
}

#[test]
fn packed_str_is_a_fixed_size_typed_view_with_value_semantics() {
    assert_eq!(
        core::mem::size_of::<PackedStr<'static, Oct, 3>>(),
        core::mem::size_of::<BitStr<'static>>()
    );
    assert_eq!(
        core::mem::size_of::<PackedStr<'static, WideCode, 7>>(),
        core::mem::size_of::<BitStr<'static>>()
    );

    let left = packed_as::<Oct, 3>(&[0, 1, 2, 3], oct);
    let right = packed_as::<Oct, 3>(&[0, 1, 2, 3], oct);
    assert!(left.as_packed_str() == right.as_packed_str());

    let different = packed_as::<Oct, 3>(&[0, 1, 2, 4], oct);
    assert!(left.as_packed_str() != different.as_packed_str());
}

#[test]
fn packed_str_char_len_tracks_widths_boundaries_and_clamped_slices() {
    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| {
        if code == 0 {
            PackedSymbol::Zero
        } else {
            PackedSymbol::One
        }
    });
    assert_eq!(binary.as_packed_str().char_len(), 2);

    let bytes = packed_as::<SparseByte, 8>(&[0, 3, 255], |code| match code {
        0 => SparseByte::Zero,
        3 => SparseByte::Middle,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    assert_eq!(bytes.as_packed_str().char_len(), 3);

    for (codes, expected) in [
        ((0..21).map(|index| index as u8 % 8).collect::<Vec<_>>(), 21),
        ((0..22).map(|index| index as u8 % 8).collect::<Vec<_>>(), 22),
    ] {
        let string = packed_as::<Oct, 3>(&codes, oct);
        assert_eq!(string.as_packed_str().char_len(), expected);
        let sliced = string
            .as_packed_str()
            .slice(UsizeCO::checked_from_start_len(1, expected - 2).unwrap());
        assert_eq!(sliced.char_len(), expected - 2);
    }

    for length in [9, 10] {
        let codes: Vec<_> = (0..length).map(|index| (index * 7) as u8 % 128).collect();
        let string = packed_as::<WideCode, 7>(&codes, wide);
        assert_eq!(string.as_packed_str().char_len(), length);
    }

    let empty = bytes
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(99, 1).unwrap());
    assert_eq!(empty.char_len(), 0);
}

#[test]
fn packed_str_is_empty_depends_only_on_view_length() {
    let empty_binary_owner = PackedString::<PackedSymbol, 1>::new();
    let empty_binary = empty_binary_owner.as_packed_str();
    assert!(empty_binary.is_empty());
    assert_eq!(empty_binary.char_len(), 0);

    let singleton = packed_as::<SparseByte, 8>(&[0], |code| match code {
        0 => SparseByte::Zero,
        _ => unreachable!(),
    });
    assert!(!singleton.as_packed_str().is_empty());

    let string = packed_as::<Oct, 3>(&[0, 1, 2, 3, 4, 5, 6, 7], oct);
    let empty_at_end = string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(99, 1).unwrap());
    let empty_from_end = string.as_packed_str().slice_from(99);
    let empty_until_start = string.as_packed_str().slice_until(0);
    for view in [empty_at_end, empty_from_end, empty_until_start] {
        assert!(view.is_empty());
        assert_eq!(view.char_len(), 0);
        assert!(view == view.clone());
    }

    let cross_word_owner = packed_as::<Oct, 3>(&[0; 22], oct);
    let cross_word = cross_word_owner.as_packed_str();
    assert!(!cross_word.is_empty());
    assert_eq!(cross_word.char_len(), 22);
}

#[test]
fn packed_str_get_decodes_bounds_offsets_and_cross_word_codes() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(empty.as_packed_str().get(0), None);
    assert_eq!(empty.as_packed_str().get(usize::MAX), None);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.as_packed_str().get(0), Some(PackedSymbol::Zero));
    assert_eq!(binary.as_packed_str().get(1), Some(PackedSymbol::One));
    assert_eq!(binary.as_packed_str().get(2), None);

    let bytes = packed_as::<SparseByte, 8>(&[0, 3, 255], |code| match code {
        0 => SparseByte::Zero,
        3 => SparseByte::Middle,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.get(0), Some(SparseByte::Zero));
    assert_eq!(byte_view.get(2), Some(SparseByte::Maximum));
    assert_eq!(byte_view.get(3), None);
    assert_eq!(byte_view.get(usize::MAX), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string.as_packed_str();
    for (index, &code) in oct_codes.iter().enumerate() {
        assert_eq!(oct_view.get(index), Some(oct(code)));
    }
    assert_eq!(oct_view.get(oct_codes.len()), None);
    let oct_slice = oct_view.slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    for (index, &code) in oct_codes[20..24].iter().enumerate() {
        assert_eq!(oct_slice.get(index), Some(oct(code)));
    }
    assert_eq!(oct_slice.get(4), None);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string.as_packed_str();
    for (index, &code) in wide_codes.iter().enumerate() {
        assert_eq!(wide_view.get(index), Some(wide(code)));
    }
    assert_eq!(wide_view.get(wide_codes.len()), None);
    let wide_slice = wide_view.slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    for (index, &code) in wide_codes[8..13].iter().enumerate() {
        assert_eq!(wide_slice.get(index), Some(wide(code)));
    }
    assert_eq!(wide_slice.get(usize::MAX), None);
}

#[test]
fn packed_str_first_is_view_relative_and_empty_safe() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(empty.as_packed_str().first(), None);

    let binary = packed_as::<PackedSymbol, 1>(&[1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.as_packed_str().first(), Some(PackedSymbol::One));

    let bytes = packed_as::<SparseByte, 8>(&[255, 0], |code| match code {
        0 => SparseByte::Zero,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.first(), Some(SparseByte::Maximum));
    assert_eq!(byte_view.slice_from(1).first(), Some(SparseByte::Zero));
    assert_eq!(byte_view.slice_from(2).first(), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(21, 2).unwrap());
    assert_eq!(oct_view.first(), Some(oct(5)));
    assert_eq!(oct_view.first(), oct_view.clone().get(0));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(9, 2).unwrap());
    assert_eq!(wide_view.first(), Some(wide(wide_codes[9])));
}

#[test]
fn packed_str_last_is_view_relative_and_empty_safe() {
    let empty = PackedString::<PackedSymbol, 1>::new();
    assert_eq!(empty.as_packed_str().last(), None);

    let binary = packed_as::<PackedSymbol, 1>(&[0, 1], |code| match code {
        0 => PackedSymbol::Zero,
        1 => PackedSymbol::One,
        _ => unreachable!(),
    });
    assert_eq!(binary.as_packed_str().last(), Some(PackedSymbol::One));

    let bytes = packed_as::<SparseByte, 8>(&[255, 3, 0], |code| match code {
        0 => SparseByte::Zero,
        3 => SparseByte::Middle,
        255 => SparseByte::Maximum,
        _ => unreachable!(),
    });
    let byte_view = bytes.as_packed_str();
    assert_eq!(byte_view.last(), Some(SparseByte::Zero));
    assert_eq!(byte_view.slice_until(2).last(), Some(SparseByte::Middle));
    assert_eq!(byte_view.slice_until(0).last(), None);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_string = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 2).unwrap());
    assert_eq!(oct_view.last(), Some(oct(5)));
    assert_eq!(oct_view.last(), oct_view.clone().get(1));

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_string = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_string
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 2).unwrap());
    assert_eq!(wide_view.last(), Some(wide(wide_codes[9])));
    assert_eq!(wide_string.as_packed_str().slice_from(16).last(), None);
}

#[test]
fn packed_str_to_packed_string_preserves_views_and_owns_storage() {
    let empty_owner = PackedString::<PackedSymbol, 1>::new();
    let empty = empty_owner.as_packed_str().to_packed_string();
    assert!(empty.is_empty());
    assert_eq!(empty.char_len(), 0);
    assert_eq!(empty.bits().bit_len(), 0);

    let oct_codes: Vec<u8> = (0..24).map(|index| index as u8 % 8).collect();
    let oct_owner = packed_as::<Oct, 3>(&oct_codes, oct);
    let oct_view = oct_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(20, 4).unwrap());
    let oct_materialized = oct_view.to_packed_string();
    assert_eq!(oct_materialized.char_len(), 4);
    assert_eq!(
        oct_materialized.to_vec(),
        oct_codes[20..24]
            .iter()
            .copied()
            .map(oct)
            .collect::<Vec<_>>()
    );
    assert!(oct_materialized.as_packed_str() == oct_view);

    let wide_codes: Vec<u8> = (0..16).map(|index| (index * 11) as u8 % 128).collect();
    let wide_owner = packed_as::<WideCode, 7>(&wide_codes, wide);
    let wide_view = wide_owner
        .as_packed_str()
        .slice(UsizeCO::checked_from_start_len(8, 5).unwrap());
    let wide_materialized = wide_view.to_packed_string();
    assert!(wide_materialized.as_packed_str() == wide_view);
    assert_eq!(wide_materialized.bits().bit_len(), 35);

    let sparse_materialized = {
        let owner = packed_as::<SparseByte, 8>(&[255, 3, 0], |code| match code {
            0 => SparseByte::Zero,
            3 => SparseByte::Middle,
            255 => SparseByte::Maximum,
            _ => unreachable!(),
        });
        owner
            .as_packed_str()
            .slice(UsizeCO::checked_from_start_len(1, 2).unwrap())
            .to_packed_string()
    };
    assert_eq!(
        sparse_materialized.to_vec(),
        vec![SparseByte::Middle, SparseByte::Zero]
    );
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
