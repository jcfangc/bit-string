use super::{Oct, PackedSymbol, SparseByte, Symbol, WideCode, oct, packed, packed_as, wide};
use bit_string::{PackedString, traits::PackedChar};
use int_intervals::UsizeCO;

#[path = "tests_for_editing/tests_for_append_and_lifecycle.rs"]
mod tests_for_append_and_lifecycle;

#[path = "tests_for_editing/tests_for_vec_edit_semantics.rs"]
mod tests_for_vec_edit_semantics;

#[path = "tests_for_editing/tests_for_cross_word_boundaries.rs"]
mod tests_for_cross_word_boundaries;

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

#[path = "tests_for_editing/tests_for_replacement.rs"]
mod tests_for_replacement;

#[path = "tests_for_editing/tests_for_slicing.rs"]
mod tests_for_slicing;

#[path = "tests_for_editing/tests_for_sequence_operations.rs"]
mod tests_for_sequence_operations;
