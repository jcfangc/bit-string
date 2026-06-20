use alloc::string::ToString;

use crate::{BitString, BitStringLenMismatch};

fn assert_or_variants(lhs: &BitString, rhs: &BitString, expected: &BitString) {
    let owned = lhs.or(rhs).unwrap();

    let mut assigned = lhs.clone();
    assigned.or_assign(rhs).unwrap();

    let into = lhs.clone().or(rhs).unwrap();

    assert_eq!(owned, *expected);
    assert_eq!(assigned, *expected);
    assert_eq!(into, *expected);

    assert_eq!(owned, assigned);
    assert_eq!(owned, into);
}

#[test]
fn computes_bitwise_or_for_same_len_inputs() {
    let lhs = BitString::try_from("101101").unwrap();
    let rhs = BitString::try_from("110011").unwrap();
    let expected = BitString::try_from("111111").unwrap();

    assert_or_variants(&lhs, &rhs, &expected);
}

#[test]
fn zeros_are_identity_for_or() {
    let lhs = BitString::try_from("101001101").unwrap();
    let rhs = BitString::zeros(lhs.bit_len());

    assert_or_variants(&lhs, &rhs, &lhs);
}

#[test]
fn ones_absorb_for_or() {
    let lhs = BitString::try_from("101001101").unwrap();
    let rhs = BitString::ones(lhs.bit_len());

    assert_or_variants(&lhs, &rhs, &rhs);

    let result = lhs.or(&rhs).unwrap();
    assert_eq!(result.count_zeros(), 0);
}

#[test]
fn works_across_word_boundaries() {
    let mut lhs = BitString::zeros(130);
    let mut rhs = BitString::zeros(130);
    let mut expected = BitString::zeros(130);

    for index in [0, 63, 64, 65, 129] {
        lhs.set(index, true);
    }

    for index in [63, 64, 128, 129] {
        rhs.set(index, true);
    }

    for index in [0, 63, 64, 65, 128, 129] {
        expected.set(index, true);
    }

    assert_or_variants(&lhs, &rhs, &expected);

    let result = lhs.or(&rhs).unwrap();

    assert_eq!(result.bit_len(), 130);
    assert_eq!(result.count_ones(), 6);

    assert_eq!(result.get(0), Some(true));
    assert_eq!(result.get(63), Some(true));
    assert_eq!(result.get(64), Some(true));
    assert_eq!(result.get(65), Some(true));
    assert_eq!(result.get(128), Some(true));
    assert_eq!(result.get(129), Some(true));
}

#[test]
fn or_does_not_mutate_inputs() {
    let lhs = BitString::try_from("101100").unwrap();
    let rhs = BitString::try_from("010011").unwrap();

    let lhs_before = lhs.clone();
    let rhs_before = rhs.clone();

    let result = lhs.or(&rhs).unwrap();

    assert_eq!(result.to_string(), "111111");
    assert_eq!(lhs, lhs_before);
    assert_eq!(rhs, rhs_before);
}

#[test]
fn or_assign_mutates_lhs_only() {
    let mut lhs = BitString::try_from("101100").unwrap();
    let rhs = BitString::try_from("010011").unwrap();
    let rhs_before = rhs.clone();

    lhs.or_assign(&rhs).unwrap();

    assert_eq!(lhs.to_string(), "111111");
    assert_eq!(rhs, rhs_before);
}

#[test]
fn or_into_matches_or() {
    let lhs = BitString::try_from("101100").unwrap();
    let rhs = BitString::try_from("010011").unwrap();

    let expected = lhs.or(&rhs).unwrap();
    let actual = lhs.or(&rhs).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn returns_len_mismatch_for_different_lengths() {
    let lhs = BitString::try_from("101").unwrap();
    let rhs = BitString::try_from("1010").unwrap();

    let expected = BitStringLenMismatch {
        lhs_len: 3,
        rhs_len: 4,
    };

    assert_eq!(lhs.or(&rhs).unwrap_err(), expected);

    let mut assigned = lhs.clone();
    assert_eq!(assigned.or_assign(&rhs).unwrap_err(), expected);
    assert_eq!(assigned.to_string(), "101");

    assert_eq!(lhs.or(&rhs).unwrap_err(), expected);
}
