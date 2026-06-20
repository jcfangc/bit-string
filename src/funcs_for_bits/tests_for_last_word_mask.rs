use super::*;
use crate::WORD_BITS;

#[test]
fn returns_full_mask_when_len_is_word_aligned() {
    assert_eq!(last_word_mask(0), u64::MAX);
    assert_eq!(last_word_mask(WORD_BITS), u64::MAX);
    assert_eq!(last_word_mask(WORD_BITS * 2), u64::MAX);
}

#[test]
fn keeps_exact_low_bits_for_partial_last_word() {
    let cases = [
        (1, 0b1),
        (2, 0b11),
        (3, 0b111),
        (WORD_BITS - 1, u64::MAX >> 1),
        (WORD_BITS + 1, 0b1),
        (WORD_BITS * 2 - 1, u64::MAX >> 1),
    ];

    for (len, expected) in cases {
        assert_eq!(last_word_mask(len), expected, "len={len}");
    }
}

#[test]
fn mask_has_no_high_bits_above_remainder() {
    let lens = [1, 7, 31, 63, 65, 79, 127];

    for len in lens {
        let rem = len % WORD_BITS;
        let mask = last_word_mask(len);

        assert_ne!(rem, 0, "test data should only contain partial words");
        assert_eq!(mask.count_ones() as usize, rem, "len={len}");
        assert_eq!(mask >> rem, 0, "len={len}");
    }
}
