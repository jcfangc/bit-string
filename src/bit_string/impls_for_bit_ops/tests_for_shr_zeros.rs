use alloc::string::ToString;

use crate::BitString;

#[test]
fn shifting_by_zero_returns_same_bits() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.shr_zeros(0);

    assert_eq!(result, bits);
}

#[test]
fn shifts_bits_right_and_fills_high_bits_with_zero() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.shr_zeros(2);

    assert_eq!(result.to_string(), "100100");
}

#[test]
fn shifting_empty_stays_empty() {
    let bits = BitString::new();

    let result = bits.shr_zeros(3);

    assert!(result.is_empty());
    assert_eq!(result.to_string(), "");
}

#[test]
fn shifting_by_len_returns_all_zeros() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.shr_zeros(bits.len());

    assert_eq!(result.len(), bits.len());
    assert_eq!(result.to_string(), "000000");
    assert_eq!(result.count_ones(), 0);
}

#[test]
fn shifting_beyond_len_returns_all_zeros() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.shr_zeros(bits.len() + 1);

    assert_eq!(result.len(), bits.len());
    assert_eq!(result.to_string(), "000000");
    assert_eq!(result.count_ones(), 0);
}

#[test]
fn works_across_bit_boundary() {
    let mut bits = BitString::zeros(130);

    for index in [0, 63, 64, 65, 129] {
        bits.set(index, true);
    }

    let result = bits.shr_zeros(1);

    assert_eq!(result.len(), 130);
    assert_eq!(result.count_ones(), 4);

    assert_eq!(result.get(0), Some(false));
    assert_eq!(result.get(62), Some(true));
    assert_eq!(result.get(63), Some(true));
    assert_eq!(result.get(64), Some(true));
    assert_eq!(result.get(65), Some(false));
    assert_eq!(result.get(128), Some(true));
    assert_eq!(result.get(129), Some(false));
}

#[test]
fn works_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    for index in [0, 63, 64, 65, 129] {
        bits.set(index, true);
    }

    let result = bits.shr_zeros(64);

    assert_eq!(result.len(), 130);
    assert_eq!(result.count_ones(), 3);

    assert_eq!(result.get(0), Some(true));
    assert_eq!(result.get(1), Some(true));
    assert_eq!(result.get(64), Some(false));
    assert_eq!(result.get(65), Some(true));
    assert_eq!(result.get(66), Some(false));
    assert_eq!(result.get(129), Some(false));
}

#[test]
fn works_across_word_and_bit_boundary() {
    let mut bits = BitString::zeros(130);

    for index in [0, 63, 64, 65, 129] {
        bits.set(index, true);
    }

    let result = bits.shr_zeros(65);

    assert_eq!(result.len(), 130);
    assert_eq!(result.count_ones(), 2);

    assert_eq!(result.get(0), Some(true));
    assert_eq!(result.get(1), Some(false));
    assert_eq!(result.get(63), Some(false));
    assert_eq!(result.get(64), Some(true));
    assert_eq!(result.get(65), Some(false));
    assert_eq!(result.get(129), Some(false));
}

#[test]
fn masks_unused_bits_in_partial_last_word() {
    let bits = BitString::ones(65);

    let result = bits.shr_zeros(1);

    assert_eq!(result.len(), 65);
    assert_eq!(result.count_ones(), 64);
    assert_eq!(result.get(63), Some(true));
    assert_eq!(result.get(64), Some(false));
    assert_eq!(result.get(65), None);
}
