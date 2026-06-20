use alloc::string::ToString;

use crate::BitString;

#[test]
fn repeat_false_with_zero_len_builds_empty_bit_string() {
    let bits = BitString::repeat(false, 0);

    assert_eq!(bits.bit_len(), 0);
    assert!(bits.is_empty());
    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 0);
    assert_eq!(bits.to_string(), "");
}

#[test]
fn repeat_true_with_zero_len_builds_empty_bit_string() {
    let bits = BitString::repeat(true, 0);

    assert_eq!(bits.bit_len(), 0);
    assert!(bits.is_empty());
    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 0);
    assert_eq!(bits.to_string(), "");
}

#[test]
fn repeat_false_fills_all_bits_with_zero() {
    for len in [1, 2, 63, 64, 65, 127, 128, 129] {
        let bits = BitString::repeat(false, len);

        assert_eq!(bits.bit_len(), len, "len={len}");
        assert_eq!(bits.count_ones(), 0, "len={len}");
        assert_eq!(bits.count_zeros(), len, "len={len}");

        for index in [0, len / 2, len - 1] {
            assert_eq!(bits.get(index), Some(false), "len={len}, index={index}");
        }

        assert_eq!(bits.get(len), None, "len={len}");
    }
}

#[test]
fn repeat_true_fills_all_bits_with_one() {
    for len in [1, 2, 63, 64, 65, 127, 128, 129] {
        let bits = BitString::repeat(true, len);

        assert_eq!(bits.bit_len(), len, "len={len}");
        assert_eq!(bits.count_ones(), len, "len={len}");
        assert_eq!(bits.count_zeros(), 0, "len={len}");

        for index in [0, len / 2, len - 1] {
            assert_eq!(bits.get(index), Some(true), "len={len}, index={index}");
        }

        assert_eq!(bits.get(len), None, "len={len}");
    }
}

#[test]
fn zeros_is_repeat_false() {
    for len in [0, 1, 63, 64, 65, 129] {
        assert_eq!(BitString::zeros(len), BitString::repeat(false, len));
    }
}

#[test]
fn ones_is_repeat_true() {
    for len in [0, 1, 63, 64, 65, 129] {
        assert_eq!(BitString::ones(len), BitString::repeat(true, len));
    }
}

#[test]
fn repeat_true_masks_unused_bits_in_last_word() {
    for len in [1, 2, 7, 31, 63, 65, 79, 127, 129] {
        let bits = BitString::repeat(true, len);

        assert_eq!(bits.count_ones(), len, "len={len}");
        assert_eq!(bits.not().count_ones(), 0, "len={len}");
    }
}

#[test]
fn repeat_true_display_matches_expected_bits() {
    let bits = BitString::repeat(true, 5);

    assert_eq!(bits.to_string(), "11111");
}

#[test]
fn repeat_false_display_matches_expected_bits() {
    let bits = BitString::repeat(false, 5);

    assert_eq!(bits.to_string(), "00000");
}
