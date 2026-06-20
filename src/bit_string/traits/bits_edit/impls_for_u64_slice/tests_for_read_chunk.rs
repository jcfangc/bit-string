use super::*;
use crate::WORD_BITS;

#[test]
fn returns_word_when_bit_start_is_word_aligned() {
    let src = [0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210];

    assert_eq!(src.read_word_at(0), src[0]);
    assert_eq!(src.read_word_at(WORD_BITS), src[1]);
}

#[test]
fn returns_zero_when_aligned_start_is_past_source() {
    let src = [0x0123_4567_89ab_cdef];

    assert_eq!(src.read_word_at(WORD_BITS), 0);
    assert_eq!(src.read_word_at(WORD_BITS * 2), 0);
}

#[test]
fn reads_shifted_chunk_within_single_word() {
    let src = [0b1011_0100u64];

    assert_eq!(src.read_word_at(2), 0b10_1101);
    assert_eq!(src.read_word_at(4), 0b1011);
}

#[test]
fn reads_chunk_across_word_boundary() {
    let src = [1u64 << (WORD_BITS - 1), 0b1010u64];

    assert_eq!(src.read_word_at(WORD_BITS - 1), 0b10101);
}

#[test]
fn treats_missing_high_word_as_zero() {
    let src = [1u64 << (WORD_BITS - 1)];

    assert_eq!(src.read_word_at(WORD_BITS - 1), 1);
}

#[test]
fn preserves_full_chunk_layout_across_boundary() {
    let src = [0xffff_ffff_ffff_0000u64, 0x0000_0000_0000_00abu64];

    let shift = 16;
    let expected = (src[0] >> shift) | (src[1] << (WORD_BITS - shift));

    assert_eq!(src.read_word_at(shift), expected);
}

#[test]
fn reads_from_later_word_with_unaligned_start() {
    let src = [0, 0b1111_0000u64, 0b1010u64];

    let expected = (src[1] >> 4) | (src[2] << (WORD_BITS - 4));

    assert_eq!(src.read_word_at(WORD_BITS + 4), expected);
}
