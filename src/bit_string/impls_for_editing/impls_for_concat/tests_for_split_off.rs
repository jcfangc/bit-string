use crate::BitString;
use alloc::string::ToString;

#[test]
fn splits_empty_bit_string_at_zero() {
    let mut bits = BitString::new();

    let rhs = bits.split_off(0);

    assert_eq!(bits.bit_len(), 0);
    assert_eq!(rhs.bit_len(), 0);
    assert_eq!(bits.to_string(), "");
    assert_eq!(rhs.to_string(), "");
}

#[test]
fn split_at_zero_moves_all_bits_to_rhs() {
    let mut bits = BitString::try_from("101001").unwrap();

    let rhs = bits.split_off(0);

    assert_eq!(bits.bit_len(), 0);
    assert_eq!(rhs.bit_len(), 6);
    assert_eq!(bits.to_string(), "");
    assert_eq!(rhs.to_string(), "101001");
}

#[test]
fn split_at_len_returns_empty_rhs() {
    let mut bits = BitString::try_from("101001").unwrap();

    let rhs = bits.split_off(bits.bit_len());

    assert_eq!(bits.bit_len(), 6);
    assert_eq!(rhs.bit_len(), 0);
    assert_eq!(bits.to_string(), "101001");
    assert_eq!(rhs.to_string(), "");
}

#[test]
fn splits_in_middle() {
    let mut bits = BitString::try_from("101001").unwrap();

    let rhs = bits.split_off(3);

    assert_eq!(bits.bit_len(), 3);
    assert_eq!(rhs.bit_len(), 3);
    assert_eq!(bits.to_string(), "101");
    assert_eq!(rhs.to_string(), "001");
}

#[test]
fn splits_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let rhs = bits.split_off(64);

    assert_eq!(bits.bit_len(), 64);
    assert_eq!(rhs.bit_len(), 66);

    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(62), Some(false));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), None);

    assert_eq!(rhs.get(0), Some(true));
    assert_eq!(rhs.get(1), Some(true));
    assert_eq!(rhs.get(2), Some(false));
    assert_eq!(rhs.get(64), Some(false));
    assert_eq!(rhs.get(65), Some(true));
    assert_eq!(rhs.get(66), None);
}

#[test]
fn split_result_is_independent_from_original() {
    let mut bits = BitString::try_from("101001").unwrap();

    let mut rhs = bits.split_off(3);

    bits.set(0, false);
    rhs.set(0, true);

    assert_eq!(bits.to_string(), "001");
    assert_eq!(rhs.to_string(), "101");
}

#[test]
fn returns_empty_when_index_gt_len() {
    let mut bits = BitString::try_from("101").unwrap();

    let rhs = bits.split_off(4);

    assert_eq!(bits.bit_len(), 3);
    assert_eq!(bits.to_string(), "101");
    assert_eq!(rhs.bit_len(), 0);
    assert_eq!(rhs.to_string(), "");
}
