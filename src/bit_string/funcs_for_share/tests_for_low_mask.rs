use crate::WORD_BITS;
use super::low_mask;

#[test]
fn returns_zero_for_zero_bits() {
    assert_eq!(low_mask(0), 0);
}

#[test]
fn returns_low_bits_for_small_widths() {
    let cases = [(1, 0b1), (2, 0b11), (3, 0b111), (4, 0b1111)];

    for (bits, expected) in cases {
        assert_eq!(low_mask(bits), expected, "bits={bits}");
    }
}

#[test]
fn returns_all_ones_for_full_word_width() {
    assert_eq!(low_mask(WORD_BITS), u64::MAX);
}

#[test]
fn clears_only_the_top_bit_for_word_bits_minus_one() {
    assert_eq!(low_mask(WORD_BITS - 1), u64::MAX >> 1);
}

#[test]
fn mask_contains_exactly_requested_number_of_low_ones() {
    let widths = [0, 1, 7, 31, WORD_BITS - 1, WORD_BITS];

    for bits in widths {
        let mask = low_mask(bits);

        assert_eq!(mask.count_ones() as usize, bits, "bits={bits}");

        if bits < WORD_BITS {
            assert_eq!(mask >> bits, 0, "bits={bits}");
        }
    }
}
