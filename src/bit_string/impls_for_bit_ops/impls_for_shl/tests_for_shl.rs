use alloc::string::ToString;

use crate::BitString;

fn assert_shl_variants(input: &BitString, amount: usize, expected: &BitString) {
    let owned = input.shl(amount);

    let mut assigned = input.clone();
    assigned.shl_assign(amount);

    let into = input.clone().shl_into(amount);

    assert_eq!(owned, *expected);
    assert_eq!(assigned, *expected);
    assert_eq!(into, *expected);

    assert_eq!(owned, assigned);
    assert_eq!(owned, into);
}

#[test]
fn shifting_empty_bit_string_keeps_it_empty() {
    let bits = BitString::new();

    let out = bits.shl(3);

    assert_eq!(out.bit_len(), 0);
    assert!(out.is_empty());
}

#[test]
fn shifting_by_zero_returns_same_bits() {
    let bits = BitString::try_from("101001").unwrap();

    let out = bits.shl(0);

    assert_eq!(out.to_string(), "101001");
}

#[test]
fn shifts_left_with_zero_fill() {
    let bits = BitString::try_from("101001").unwrap();

    let out = bits.shl(2);

    assert_eq!(out.to_string(), "001010");
}

#[test]
fn shifting_by_len_or_more_returns_all_zeros() {
    let bits = BitString::try_from("101001").unwrap();

    let by_len = bits.shl(bits.bit_len());
    let beyond_len = bits.shl(bits.bit_len() + 1);

    assert_eq!(by_len.to_string(), "000000");
    assert_eq!(beyond_len.to_string(), "000000");
}

#[test]
fn shifts_across_bit_boundary_inside_word() {
    let mut bits = BitString::zeros(16);

    bits.set(0, true);
    bits.set(3, true);
    bits.set(7, true);

    let out = bits.shl(5);

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

    let out = bits.shl(64);

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

    let out = bits.shl(1);

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

    let out = bits.shl(1);

    assert_eq!(out.bit_len(), 65);
    assert_eq!(out.get(63), Some(false));
    assert_eq!(out.get(64), Some(true));
    assert_eq!(out.get(65), None);
    assert_eq!(out.count_ones(), 1);
}

#[test]
fn shift_variants_match_for_in_word_shift() {
    let bits = BitString::try_from("101001").unwrap();
    let expected = BitString::try_from("001010").unwrap();

    assert_shl_variants(&bits, 2, &expected);
}

#[test]
fn shift_variants_match_across_word_boundary() {
    let mut bits = BitString::zeros(130);
    let mut expected = BitString::zeros(130);

    for index in [0, 63, 64] {
        bits.set(index, true);
    }

    for index in [1, 64, 65] {
        expected.set(index, true);
    }

    assert_shl_variants(&bits, 1, &expected);
}

#[test]
fn shift_variants_match_when_shift_clears_all_bits() {
    let bits = BitString::try_from("101001").unwrap();
    let expected = BitString::zeros(bits.bit_len());

    assert_shl_variants(&bits, bits.bit_len(), &expected);
    assert_shl_variants(&bits, bits.bit_len() + 1, &expected);
}
