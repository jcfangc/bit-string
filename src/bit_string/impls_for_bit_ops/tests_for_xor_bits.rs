use alloc::string::ToString;

use crate::{BitString, BitStringLenMismatch};

#[test]
fn xors_equal_length_bit_strings() {
    let lhs = BitString::try_from("101001").unwrap();
    let rhs = BitString::try_from("110010").unwrap();

    let out = lhs.xor_bits(&rhs).unwrap();

    assert_eq!(out.to_string(), "011011");
}

#[test]
fn xor_with_self_returns_all_zeros() {
    let bits = BitString::try_from("101001").unwrap();

    let out = bits.xor_bits(&bits).unwrap();

    assert_eq!(out.to_string(), "000000");
    assert_eq!(out.count_ones(), 0);
}

#[test]
fn xor_with_zeros_returns_original_bits() {
    let bits = BitString::try_from("101001").unwrap();
    let zeros = BitString::zeros(bits.len());

    let out = bits.xor_bits(&zeros).unwrap();

    assert_eq!(out, bits);
}

#[test]
fn returns_len_mismatch_error() {
    let lhs = BitString::try_from("101").unwrap();
    let rhs = BitString::try_from("1010").unwrap();

    let err = lhs.xor_bits(&rhs).unwrap_err();

    assert_eq!(
        err,
        BitStringLenMismatch {
            lhs_len: 3,
            rhs_len: 4,
        }
    );
}

#[test]
fn works_across_word_boundaries() {
    let mut lhs = BitString::zeros(130);
    let mut rhs = BitString::zeros(130);

    lhs.set(0, true);
    lhs.set(63, true);
    lhs.set(64, true);
    lhs.set(129, true);

    rhs.set(1, true);
    rhs.set(63, true);
    rhs.set(65, true);
    rhs.set(129, true);

    let out = lhs.xor_bits(&rhs).unwrap();

    assert_eq!(out.get(0), Some(true));
    assert_eq!(out.get(1), Some(true));
    assert_eq!(out.get(62), Some(false));
    assert_eq!(out.get(63), Some(false));
    assert_eq!(out.get(64), Some(true));
    assert_eq!(out.get(65), Some(true));
    assert_eq!(out.get(128), Some(false));
    assert_eq!(out.get(129), Some(false));
    assert_eq!(out.get(130), None);
}

#[test]
fn masks_unused_tail_bits() {
    let lhs = BitString::ones(65);
    let rhs = BitString::zeros(65);

    let out = lhs.xor_bits(&rhs).unwrap();

    assert_eq!(out.len(), 65);
    assert_eq!(out.count_ones(), 65);
    assert_eq!(out.get(64), Some(true));
    assert_eq!(out.get(65), None);
}
