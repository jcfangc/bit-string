use alloc::string::ToString;

use crate::BitString;

#[test]
fn shifting_empty_bit_string_keeps_it_empty() {
    let bits = BitString::new();

    let out = bits.shl_zeros(3);

    assert_eq!(out.len(), 0);
    assert!(out.is_empty());
}

#[test]
fn shifting_by_zero_returns_same_bits() {
    let bits = BitString::try_from("101001").unwrap();

    let out = bits.shl_zeros(0);

    assert_eq!(out.to_string(), "101001");
}

#[test]
fn shifts_left_with_zero_fill() {
    let bits = BitString::try_from("101001").unwrap();

    let out = bits.shl_zeros(2);

    assert_eq!(out.to_string(), "001010");
}

#[test]
fn shifting_by_len_or_more_returns_all_zeros() {
    let bits = BitString::try_from("101001").unwrap();

    let by_len = bits.shl_zeros(bits.len());
    let beyond_len = bits.shl_zeros(bits.len() + 1);

    assert_eq!(by_len.to_string(), "000000");
    assert_eq!(beyond_len.to_string(), "000000");
}

#[test]
fn shifts_across_bit_boundary_inside_word() {
    let mut bits = BitString::zeros(16);

    bits.set(0, true);
    bits.set(3, true);
    bits.set(7, true);

    let out = bits.shl_zeros(5);

    assert_eq!(out.get(0), Some(false));
    assert_eq!(out.get(4), Some(false));
    assert_eq!(out.get(5), Some(true));
    assert_eq!(out.get(8), Some(true));
    assert_eq!(out.get(12), Some(true));
    assert_eq!(out.get(13), Some(false));
}

#[test]
fn shifts_by_exact_word_size() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(1, true);
    bits.set(63, true);
    bits.set(64, true);

    let out = bits.shl_zeros(64);

    assert_eq!(out.get(0), Some(false));
    assert_eq!(out.get(63), Some(false));
    assert_eq!(out.get(64), Some(true));
    assert_eq!(out.get(65), Some(true));
    assert_eq!(out.get(127), Some(true));
    assert_eq!(out.get(128), Some(true));
    assert_eq!(out.get(129), Some(false));
    assert_eq!(out.get(130), None);
}

#[test]
fn shifts_across_word_boundary_with_bit_offset() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);

    let out = bits.shl_zeros(1);

    assert_eq!(out.get(0), Some(false));
    assert_eq!(out.get(1), Some(true));
    assert_eq!(out.get(63), Some(false));
    assert_eq!(out.get(64), Some(true));
    assert_eq!(out.get(65), Some(true));
    assert_eq!(out.get(66), Some(false));
}

#[test]
fn masks_bits_shifted_beyond_logical_len() {
    let mut bits = BitString::zeros(65);

    bits.set(63, true);
    bits.set(64, true);

    let out = bits.shl_zeros(1);

    assert_eq!(out.len(), 65);
    assert_eq!(out.get(63), Some(false));
    assert_eq!(out.get(64), Some(true));
    assert_eq!(out.get(65), None);
    assert_eq!(out.count_ones(), 1);
}
