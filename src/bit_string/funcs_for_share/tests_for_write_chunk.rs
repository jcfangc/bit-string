use super::write_chunk;
use crate::bit_string::WORD_BITS;

#[test]
fn writes_aligned_chunk_into_single_word() {
    let mut dst = [0u64; 2];

    write_chunk(&mut dst, 0, 0b1011, 4);

    assert_eq!(dst[0], 0b1011);
    assert_eq!(dst[1], 0);
}

#[test]
fn masks_value_to_requested_len() {
    let mut dst = [0u64; 1];

    write_chunk(&mut dst, 4, u64::MAX, 3);

    assert_eq!(dst[0], 0b111 << 4);
}

#[test]
fn writes_unaligned_chunk_inside_single_word() {
    let mut dst = [0u64; 1];

    write_chunk(&mut dst, 5, 0b10101, 5);

    assert_eq!(dst[0], 0b10101 << 5);
}

#[test]
fn writes_chunk_across_word_boundary() {
    let mut dst = [0u64; 2];

    write_chunk(&mut dst, WORD_BITS - 2, 0b1011, 4);

    assert_eq!(dst[0], 0b11u64 << (WORD_BITS - 2));
    assert_eq!(dst[1], 0b10);
}

#[test]
fn does_not_write_past_dst_when_crossing_boundary_without_next_word() {
    let mut dst = [0u64; 1];

    write_chunk(&mut dst, WORD_BITS - 2, 0b1011, 4);

    assert_eq!(dst[0], 0b11u64 << (WORD_BITS - 2));
}

#[test]
fn ors_into_existing_bits_instead_of_overwriting() {
    let mut dst = [0b1000u64];

    write_chunk(&mut dst, 0, 0b0011, 2);

    assert_eq!(dst[0], 0b1011);
}

#[test]
fn zero_len_writes_nothing() {
    let mut dst = [0b1010u64];

    write_chunk(&mut dst, 1, u64::MAX, 0);

    assert_eq!(dst[0], 0b1010);
}

#[test]
fn full_word_len_preserves_all_value_bits() {
    let mut dst = [0u64; 1];

    write_chunk(&mut dst, 0, u64::MAX, WORD_BITS);

    assert_eq!(dst[0], u64::MAX);
}
