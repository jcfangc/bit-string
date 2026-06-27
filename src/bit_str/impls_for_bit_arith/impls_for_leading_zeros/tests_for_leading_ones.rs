use int_interval::UsizeCO;

use crate::BitString;

/// An empty view has no leading ones.
#[test]
fn empty_view_returns_zero() {
    let bits = BitString::try_from("01010").unwrap();
    let v = bits.as_bit_str().slice(UsizeCO::try_new(10, 20).unwrap());
    assert_eq!(v.leading_ones(), 0);
}

/// Starts with a 0 → zero leading ones.
#[test]
fn starts_with_zero() {
    let bits = BitString::try_from("01101").unwrap();
    assert_eq!(bits.as_bit_str().leading_ones(), 0);
}

/// Starts with several ones.
#[test]
fn leading_one_run() {
    let bits = BitString::try_from("11010").unwrap();
    assert_eq!(bits.as_bit_str().leading_ones(), 2);
}

/// Single one bit.
#[test]
fn single_one() {
    let bits = BitString::ones(1);
    assert_eq!(bits.as_bit_str().leading_ones(), 1);
}

/// Single zero bit.
#[test]
fn single_zero() {
    let bits = BitString::zeros(1);
    assert_eq!(bits.as_bit_str().leading_ones(), 0);
}

/// All ones at various lengths including word-boundary values.
#[test]
fn all_ones_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::ones(len);
        assert_eq!(bits.as_bit_str().leading_ones(), len, "len={len}");
    }
}

/// All zeros at various lengths.
#[test]
fn all_zeros_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::zeros(len);
        assert_eq!(bits.as_bit_str().leading_ones(), 0, "len={len}");
    }
}

/// First 0 across word boundary.
#[test]
fn first_zero_in_second_word() {
    let mut bits = BitString::ones(130);
    bits.set(100, false);
    assert_eq!(bits.as_bit_str().leading_ones(), 100);
}

/// First 0 exactly at word boundary.
#[test]
fn first_zero_at_word_boundary() {
    let mut bits = BitString::ones(130);
    bits.set(64, false);
    assert_eq!(bits.as_bit_str().leading_ones(), 64);
}

/// Unaligned view.
#[test]
fn unaligned_start() {
    let mut bits = BitString::ones(130);
    bits.set(10, false); // first zero at bit 10
    let v = bits.as_bit_str().slice(UsizeCO::try_new(3, 130).unwrap());
    assert_eq!(v.leading_ones(), 7); // from bit 3, ones at positions 3..=9, then 0 at 10
}

/// Unaligned view all ones.
#[test]
fn unaligned_all_ones() {
    let bits = BitString::ones(200);
    let v = bits.as_bit_str().slice(UsizeCO::try_new(3, 130).unwrap());
    assert_eq!(v.leading_ones(), 127);
}

/// Unaligned single word.
#[test]
fn unaligned_single_word() {
    let bits = BitString::try_from("00000111").unwrap();
    // bits: 0 0 0 0 0 1 1 1

    // View bits 4..8 → "0111"
    let v = bits.as_bit_str().slice(UsizeCO::try_new(4, 8).unwrap());
    assert_eq!(v.leading_ones(), 0);

    // View bits 5..8 → "111"
    let v = bits.as_bit_str().slice(UsizeCO::try_new(5, 8).unwrap());
    assert_eq!(v.leading_ones(), 3);
}

/// Last bit is zero.
#[test]
fn last_bit_is_zero() {
    let mut bits = BitString::ones(130);
    bits.set(129, false);
    assert_eq!(bits.as_bit_str().leading_ones(), 129);
}

/// Invariant: leading_ones ≤ bit_len and leading_ones == bit_len iff all ones.
#[test]
fn invariant_leading_ones_bounds() {
    let mut bits = BitString::ones(200);
    for i in (1..200).step_by(7) {
        bits.set(i, false);
    }
    let full = bits.as_bit_str();

    for start in [0, 1, 5, 63, 64, 65, 127, 128] {
        for len in [10, 63, 64, 65, 128, 129] {
            let end = (start + len).min(full.bit_len());
            if start == end {
                continue;
            }
            let v = full.slice(UsizeCO::try_new(start, end).unwrap());
            let lo = v.leading_ones();
            assert!(lo <= v.bit_len(), "start={start} end={end} lo={lo}");
            if v.is_all_ones() {
                assert_eq!(lo, v.bit_len(), "all ones: start={start} end={end}");
            } else {
                assert_eq!(v.get(lo), Some(false), "start={start} end={end} lo={lo}");
            }
        }
    }
}
