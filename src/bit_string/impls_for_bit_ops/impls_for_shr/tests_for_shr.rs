use alloc::string::ToString;

use crate::BitString;

fn assert_shr_variants(input: &BitString, amount: usize, expected: &BitString) {
    let owned = input.shr(amount);

    let mut assigned = input.clone();
    assigned.shr_assign(amount);

    let into = input.clone().shr_into(amount);

    assert_eq!(owned, *expected);
    assert_eq!(assigned, *expected);
    assert_eq!(into, *expected);

    assert_eq!(owned, assigned);
    assert_eq!(owned, into);
}

#[test]
fn shifting_empty_bit_string_keeps_it_empty() {
    let bits = BitString::new();

    let out = bits.shr(3);

    assert_eq!(out.bit_len(), 0);
    assert!(out.is_empty());
}

#[test]
fn shifting_by_zero_returns_same_bits() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.shr(0);

    assert_eq!(result, bits);
}

#[test]
fn shifts_bits_right_and_fills_high_bits_with_zero() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.shr(2);

    assert_eq!(result.to_string(), "100100");
}

#[test]
fn shifting_by_len_returns_all_zeros() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.shr(bits.bit_len());

    assert_eq!(result.bit_len(), bits.bit_len());
    assert_eq!(result.to_string(), "000000");
    assert_eq!(result.count_ones(), 0);
}

#[test]
fn shifting_beyond_len_returns_all_zeros() {
    let bits = BitString::try_from("101001").unwrap();

    let result = bits.shr(bits.bit_len() + 1);

    assert_eq!(result.bit_len(), bits.bit_len());
    assert_eq!(result.to_string(), "000000");
    assert_eq!(result.count_ones(), 0);
}

#[test]
fn works_across_bit_boundary() {
    let mut bits = BitString::zeros(130);

    for index in [0, 63, 64, 65, 129] {
        bits.set(index, true);
    }

    let result = bits.shr(1);

    assert_eq!(result.bit_len(), 130);
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

    let result = bits.shr(64);

    assert_eq!(result.bit_len(), 130);
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

    let result = bits.shr(65);

    assert_eq!(result.bit_len(), 130);
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

    let result = bits.shr(1);

    assert_eq!(result.bit_len(), 65);
    assert_eq!(result.count_ones(), 64);
    assert_eq!(result.get(63), Some(true));
    assert_eq!(result.get(64), Some(false));
    assert_eq!(result.get(65), None);
}

#[test]
fn shift_variants_match_for_in_word_shift() {
    let bits = BitString::try_from("101001").unwrap();
    let expected = BitString::try_from("100100").unwrap();

    assert_shr_variants(&bits, 2, &expected);
}

#[test]
fn shift_variants_match_across_word_boundary() {
    let mut bits = BitString::zeros(130);
    let mut expected = BitString::zeros(130);

    for index in [0, 63, 64, 65, 129] {
        bits.set(index, true);
    }

    // src[64] → dst[0], src[65] → dst[1], src[129] → dst[65]
    for index in [0, 1, 65] {
        expected.set(index, true);
    }

    assert_shr_variants(&bits, 64, &expected);
}

#[test]
fn shift_variants_match_when_shift_clears_all_bits() {
    let bits = BitString::try_from("101001").unwrap();
    let expected = BitString::zeros(bits.bit_len());

    assert_shr_variants(&bits, bits.bit_len(), &expected);
    assert_shr_variants(&bits, bits.bit_len() + 1, &expected);
}
