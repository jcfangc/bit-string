use alloc::string::ToString;

use crate::{BitString, BitStringLenMismatch};

#[test]
fn computes_bitwise_and_for_same_len_inputs() {
    let lhs = BitString::try_from("101101").unwrap();
    let rhs = BitString::try_from("110011").unwrap();

    let result = lhs.and(&rhs).unwrap();

    assert_eq!(result.to_string(), "100001");
}

#[test]
fn ones_are_identity_for_and() {
    let lhs = BitString::try_from("101001101").unwrap();
    let rhs = BitString::ones(lhs.len());

    let result = lhs.and(&rhs).unwrap();

    assert_eq!(result, lhs);
}

#[test]
fn zeros_absorb_for_and() {
    let lhs = BitString::try_from("101001101").unwrap();
    let rhs = BitString::zeros(lhs.len());

    let result = lhs.and(&rhs).unwrap();

    assert_eq!(result, rhs);
    assert_eq!(result.count_ones(), 0);
}

#[test]
fn works_across_word_boundaries() {
    let mut lhs = BitString::zeros(130);
    let mut rhs = BitString::zeros(130);

    for index in [0, 63, 64, 65, 129] {
        lhs.set(index, true);
    }

    for index in [63, 64, 128, 129] {
        rhs.set(index, true);
    }

    let result = lhs.and(&rhs).unwrap();

    assert_eq!(result.len(), 130);
    assert_eq!(result.count_ones(), 3);

    assert_eq!(result.get(0), Some(false));
    assert_eq!(result.get(63), Some(true));
    assert_eq!(result.get(64), Some(true));
    assert_eq!(result.get(65), Some(false));
    assert_eq!(result.get(128), Some(false));
    assert_eq!(result.get(129), Some(true));
}

#[test]
fn returns_len_mismatch_for_different_lengths() {
    let lhs = BitString::try_from("101").unwrap();
    let rhs = BitString::try_from("1010").unwrap();

    let err = lhs.and(&rhs).unwrap_err();

    assert_eq!(
        err,
        BitStringLenMismatch {
            lhs_len: 3,
            rhs_len: 4,
        }
    );
}
