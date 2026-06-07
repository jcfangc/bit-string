use alloc::string::ToString;

use crate::BitString;

#[test]
fn flips_each_bit() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.not_bits();

    assert_eq!(result.to_string(), "010110");
}

#[test]
fn preserves_len() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.not_bits();

    assert_eq!(result.len(), bits.len());
}

#[test]
fn empty_stays_empty() {
    let bits = BitString::new();

    let result = bits.not_bits();

    assert!(result.is_empty());
    assert_eq!(result.to_string(), "");
}

#[test]
fn double_not_returns_original() {
    let bits = BitString::try_from("00101101").unwrap();

    let result = bits.not_bits().not_bits();

    assert_eq!(result, bits);
}

#[test]
fn masks_unused_bits_in_partial_last_word() {
    let zeros = BitString::zeros(65);
    let ones = zeros.not_bits();

    assert_eq!(ones.len(), 65);
    assert_eq!(ones.count_ones(), 65);
    assert_eq!(ones.count_zeros(), 0);

    let zeros_again = ones.not_bits();

    assert_eq!(zeros_again.len(), 65);
    assert_eq!(zeros_again.count_ones(), 0);
    assert_eq!(zeros_again.count_zeros(), 65);
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(130);

    for index in [0, 63, 64, 65, 129] {
        bits.set(index, true);
    }

    let result = bits.not_bits();

    assert_eq!(result.len(), 130);
    assert_eq!(result.count_ones(), 125);

    assert_eq!(result.get(0), Some(false));
    assert_eq!(result.get(1), Some(true));
    assert_eq!(result.get(63), Some(false));
    assert_eq!(result.get(64), Some(false));
    assert_eq!(result.get(65), Some(false));
    assert_eq!(result.get(66), Some(true));
    assert_eq!(result.get(128), Some(true));
    assert_eq!(result.get(129), Some(false));
    assert_eq!(result.get(130), None);
}
