use super::{
    Oct, PackedSymbol, SparseByte, Symbol, WideCode, oct, packed, packed_as, symbol, wide,
};
use bit_string::{BitStr, PackedString, traits::PackedChar};
use int_intervals::UsizeCO;

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

#[path = "tests_for_access/tests_for_view_metadata.rs"]
mod tests_for_view_metadata;

#[path = "tests_for_access/tests_for_accessors.rs"]
mod tests_for_accessors;

#[path = "tests_for_access/tests_for_slices.rs"]
mod tests_for_slices;

#[path = "tests_for_access/tests_for_vec_access_semantics.rs"]
mod tests_for_vec_access_semantics;

#[path = "tests_for_access/tests_for_cross_word_boundaries.rs"]
mod tests_for_cross_word_boundaries;
