use int_interval::UsizeCO;

use crate::BitString;

/// An empty view has no leading zeros.
#[test]
fn empty_view_returns_zero() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str().slice(UsizeCO::try_new(10, 20).unwrap());
    assert_eq!(v.leading_zeros(), 0);
}

/// Starts with a 1 → zero leading zeros.
#[test]
fn starts_with_one() {
    let bits = BitString::try_from("10011").unwrap();
    assert_eq!(bits.as_bit_str().leading_zeros(), 0);
}

/// Starts with several zeros.
#[test]
fn leading_zero_run() {
    let bits = BitString::try_from("00101").unwrap();
    assert_eq!(bits.as_bit_str().leading_zeros(), 2);
}

/// Single zero bit.
#[test]
fn single_zero() {
    let bits = BitString::zeros(1);
    assert_eq!(bits.as_bit_str().leading_zeros(), 1);
}

/// Single one bit.
#[test]
fn single_one() {
    let bits = BitString::ones(1);
    assert_eq!(bits.as_bit_str().leading_zeros(), 0);
}

/// All zeros at various lengths including word-boundary values.
#[test]
fn all_zeros_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::zeros(len);
        assert_eq!(bits.as_bit_str().leading_zeros(), len, "len={len}");
    }
}

/// All ones at various lengths.
#[test]
fn all_ones_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::ones(len);
        assert_eq!(bits.as_bit_str().leading_zeros(), 0, "len={len}");
    }
}

/// First 1 is deep into the second word (bit 64+).
#[test]
fn first_one_in_second_word() {
    let mut bits = BitString::zeros(130);
    bits.set(100, true);
    assert_eq!(bits.as_bit_str().leading_zeros(), 100);
}

/// First 1 spans across the first two words (index 64).
#[test]
fn first_one_at_word_boundary() {
    let mut bits = BitString::zeros(130);
    bits.set(64, true);
    assert_eq!(bits.as_bit_str().leading_zeros(), 64);
}

/// Unaligned view: start is not word-aligned.
#[test]
fn unaligned_start() {
    let mut bits = BitString::zeros(130);
    // Set bit 10 to 1. View starts at bit 3, so leading zeros = 7.
    bits.set(10, true);
    let v = bits.as_bit_str().slice(UsizeCO::try_new(3, 130).unwrap());
    assert_eq!(v.leading_zeros(), 7);
}

/// Unaligned view that is all zeros.
#[test]
fn unaligned_all_zeros() {
    let bits = BitString::zeros(200);
    let v = bits.as_bit_str().slice(UsizeCO::try_new(3, 130).unwrap());
    assert_eq!(v.leading_zeros(), 127); // bit_len = 130 - 3
}

/// Unaligned single word: both start and end within the same word.
#[test]
fn unaligned_single_word() {
    let bits = BitString::try_from("11111000").unwrap();
    // bits: 1 1 1 1 1 0 0 0
    // View bits 2..6 → "1110" = bits 2,3,4,5 (all ones)
    let v = bits.as_bit_str().slice(UsizeCO::try_new(2, 6).unwrap());
    assert_eq!(v.leading_zeros(), 0);

    // View bits 5..8 → "000"
    let v = bits.as_bit_str().slice(UsizeCO::try_new(5, 8).unwrap());
    assert_eq!(v.leading_zeros(), 3);
}

/// A long view with a 1 exactly at the last bit.
#[test]
fn last_bit_is_one() {
    let mut bits = BitString::zeros(130);
    bits.set(129, true);
    assert_eq!(bits.as_bit_str().leading_zeros(), 129);
}

/// Alternating pattern starting with 0.
#[test]
fn alternating_starting_with_zero() {
    // "01010101" — 8 bits
    let bits = BitString::try_from("01010101").unwrap();
    assert_eq!(bits.as_bit_str().leading_zeros(), 1);
}

/// Invariant: leading_zeros ≤ bit_len and leading_zeros == bit_len iff all zeros.
#[test]
fn invariant_leading_zeros_bounds() {
    let mut bits = BitString::zeros(200);
    for i in (0..200).step_by(7) {
        bits.set(i, true);
    }
    let full = bits.as_bit_str();

    for start in [0, 1, 5, 63, 64, 65, 127, 128] {
        for len in [10, 63, 64, 65, 128, 129] {
            let end = (start + len).min(full.bit_len());
            if start == end {
                continue;
            }
            let v = full.slice(UsizeCO::try_new(start, end).unwrap());
            let lz = v.leading_zeros();
            assert!(lz <= v.bit_len(), "start={start} end={end} lz={lz}");
            if v.is_all_zeros() {
                assert_eq!(lz, v.bit_len(), "all zeros: start={start} end={end}");
            } else {
                // If not all zeros, the bit at position `lz` must be 1.
                assert_eq!(v.get(lz), Some(true), "start={start} end={end} lz={lz}");
            }
        }
    }
}
