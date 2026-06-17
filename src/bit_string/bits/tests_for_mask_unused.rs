use super::*;
use crate::WORD_BITS;

#[test]
fn does_nothing_for_empty_words() {
    let mut bits = [];

    bits.mask_unused_bits(7);

    assert_eq!(bits, [0u64; 0]);
}

#[test]
fn keeps_last_word_unchanged_when_len_is_word_aligned() {
    let mut bits = [0x1234_5678_9abc_def0];

    bits.mask_unused_bits(WORD_BITS);

    assert_eq!(bits, [0x1234_5678_9abc_def0]);
}

#[test]
fn masks_high_bits_in_partial_last_word() {
    let mut bits = [u64::MAX];

    bits.mask_unused_bits(3);

    assert_eq!(bits, [0b111]);
}

#[test]
fn masks_only_the_last_word() {
    let mut bits = [u64::MAX, u64::MAX];

    bits.mask_unused_bits(WORD_BITS + 5);

    assert_eq!(bits, [u64::MAX, 0b1_1111]);
}

#[test]
fn preserves_low_bits_and_clears_unused_high_bits() {
    let mut bits = [0b1011_0101];

    bits.mask_unused_bits(4);

    assert_eq!(bits, [0b0101]);
}

#[test]
fn len_zero_with_non_empty_words_uses_full_mask() {
    let mut bits = [0xdead_beef_dead_beef];

    bits.mask_unused_bits(0);

    assert_eq!(bits, [0xdead_beef_dead_beef]);
}
