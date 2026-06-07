use alloc::string::ToString;

use crate::BitString;

#[test]
fn ors_bits_with_same_length() {
    let lhs = BitString::try_from("100100").unwrap();
    let rhs = BitString::try_from("010110").unwrap();

    let out = lhs.or_bits(&rhs).unwrap();

    assert_eq!(out.to_string(), "110110");
}

#[test]
fn ors_empty_bit_strings() {
    let lhs = BitString::new();
    let rhs = BitString::new();

    let out = lhs.or_bits(&rhs).unwrap();

    assert_eq!(out.len(), 0);
    assert!(out.is_empty());
    assert_eq!(out.to_string(), "");
}

#[test]
fn rejects_different_lengths() {
    let lhs = BitString::try_from("101").unwrap();
    let rhs = BitString::try_from("1010").unwrap();

    let err = lhs.or_bits(&rhs).unwrap_err();

    assert_eq!(err.lhs_len, 3);
    assert_eq!(err.rhs_len, 4);
}

#[test]
fn ors_across_word_boundaries() {
    let mut lhs = BitString::zeros(130);
    let mut rhs = BitString::zeros(130);

    lhs.set(0, true);
    lhs.set(63, true);
    lhs.set(129, true);

    rhs.set(1, true);
    rhs.set(64, true);
    rhs.set(129, true);

    let out = lhs.or_bits(&rhs).unwrap();

    assert_eq!(out.get(0), Some(true));
    assert_eq!(out.get(1), Some(true));
    assert_eq!(out.get(62), Some(false));
    assert_eq!(out.get(63), Some(true));
    assert_eq!(out.get(64), Some(true));
    assert_eq!(out.get(65), Some(false));
    assert_eq!(out.get(128), Some(false));
    assert_eq!(out.get(129), Some(true));
    assert_eq!(out.get(130), None);
}

#[test]
fn masks_unused_high_bits_in_last_word() {
    let lhs = BitString::ones(65);
    let rhs = BitString::zeros(65);

    let out = lhs.or_bits(&rhs).unwrap();

    assert_eq!(out.count_ones(), 65);
    assert_eq!(out.as_words().len(), 2);
    assert_eq!(out.as_words()[1], 1);
}
