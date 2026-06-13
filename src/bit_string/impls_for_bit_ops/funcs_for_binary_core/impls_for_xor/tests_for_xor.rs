use alloc::string::ToString;

use crate::{BitString, BitStringLenMismatch};

fn assert_xor_variants(lhs: &BitString, rhs: &BitString, expected: &BitString) {
    let owned = lhs.xor(rhs).unwrap();

    let mut assigned = lhs.clone();
    assigned.xor_assign(rhs).unwrap();

    let into = lhs.clone().xor_into(rhs).unwrap();

    assert_eq!(owned, *expected);
    assert_eq!(assigned, *expected);
    assert_eq!(into, *expected);

    assert_eq!(owned, assigned);
    assert_eq!(owned, into);
}

#[test]
fn computes_bitwise_xor_for_same_len_inputs() {
    let lhs = BitString::try_from("101101").unwrap();
    let rhs = BitString::try_from("110011").unwrap();
    let expected = BitString::try_from("011110").unwrap();

    assert_xor_variants(&lhs, &rhs, &expected);
}

#[test]
fn zeros_are_identity_for_xor() {
    let lhs = BitString::try_from("101001101").unwrap();
    let rhs = BitString::zeros(lhs.len());

    assert_xor_variants(&lhs, &rhs, &lhs);
}

#[test]
fn self_xor_is_zero() {
    let lhs = BitString::try_from("101001101").unwrap();
    let expected = BitString::zeros(lhs.len());

    assert_xor_variants(&lhs, &lhs, &expected);

    let result = lhs.xor(&lhs).unwrap();
    assert_eq!(result.count_ones(), 0);
}

#[test]
fn ones_flip_all_bits_for_xor() {
    let lhs = BitString::try_from("101001101").unwrap();
    let rhs = BitString::ones(lhs.len());
    let expected = BitString::try_from("010110010").unwrap();

    assert_xor_variants(&lhs, &rhs, &expected);
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

    for index in [0, 65, 128] {
        expected.set(index, true);
    }

    assert_xor_variants(&lhs, &rhs, &expected);

    let result = lhs.xor(&rhs).unwrap();

    assert_eq!(result.len(), 130);
    assert_eq!(result.count_ones(), 3);

    assert_eq!(result.get(0), Some(true));
    assert_eq!(result.get(63), Some(false));
    assert_eq!(result.get(64), Some(false));
    assert_eq!(result.get(65), Some(true));
    assert_eq!(result.get(128), Some(true));
    assert_eq!(result.get(129), Some(false));
}

#[test]
fn xor_does_not_mutate_inputs() {
    let lhs = BitString::try_from("101101").unwrap();
    let rhs = BitString::try_from("110011").unwrap();

    let lhs_before = lhs.clone();
    let rhs_before = rhs.clone();

    let result = lhs.xor(&rhs).unwrap();

    assert_eq!(result.to_string(), "011110");
    assert_eq!(lhs, lhs_before);
    assert_eq!(rhs, rhs_before);
}

#[test]
fn xor_assign_mutates_lhs_only() {
    let mut lhs = BitString::try_from("101101").unwrap();
    let rhs = BitString::try_from("110011").unwrap();
    let rhs_before = rhs.clone();

    lhs.xor_assign(&rhs).unwrap();

    assert_eq!(lhs.to_string(), "011110");
    assert_eq!(rhs, rhs_before);
}

#[test]
fn xor_into_matches_xor() {
    let lhs = BitString::try_from("101101").unwrap();
    let rhs = BitString::try_from("110011").unwrap();

    let expected = lhs.xor(&rhs).unwrap();
    let actual = lhs.xor_into(&rhs).unwrap();

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

    assert_eq!(lhs.xor(&rhs).unwrap_err(), expected);

    let mut assigned = lhs.clone();
    assert_eq!(assigned.xor_assign(&rhs).unwrap_err(), expected);
    assert_eq!(assigned.to_string(), "101");

    assert_eq!(lhs.xor_into(&rhs).unwrap_err(), expected);
}
