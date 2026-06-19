use crate::BitString;
use alloc::string::ToString;

#[test]
fn truncates_to_shorter_len() {
    let mut bits = BitString::try_from("101101").unwrap();

    bits.truncate(4);

    assert_eq!(bits.bit_len(), 4);
    assert_eq!(bits.to_string(), "1011");
    assert_eq!(bits.get(4), None);
}

#[test]
fn truncates_to_zero() {
    let mut bits = BitString::try_from("101101").unwrap();

    bits.truncate(0);

    assert_eq!(bits.bit_len(), 0);
    assert!(bits.is_empty());
    assert_eq!(bits.to_string(), "");
    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 0);
}

#[test]
fn truncating_to_same_len_is_noop() {
    let mut bits = BitString::try_from("101101").unwrap();

    bits.truncate(bits.bit_len());

    assert_eq!(bits.bit_len(), 6);
    assert_eq!(bits.to_string(), "101101");
}

#[test]
fn truncates_across_word_boundary() {
    let mut bits = BitString::ones(130);

    bits.truncate(65);

    assert_eq!(bits.bit_len(), 65);
    assert_eq!(bits.count_ones(), 65);
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), None);
}

#[test]
fn masks_unused_bits_after_truncating_inside_word() {
    let mut bits = BitString::ones(70);

    bits.truncate(63);

    assert_eq!(bits.bit_len(), 63);
    assert_eq!(bits.count_ones(), 63);
    assert_eq!(bits.count_zeros(), 0);
    assert_eq!(bits.get(62), Some(true));
    assert_eq!(bits.get(63), None);
}

#[test]
fn truncate_preserves_prefix_only() {
    let mut bits = BitString::try_from("1010101111").unwrap();

    bits.truncate(6);

    assert_eq!(bits.to_string(), "101010");
}

#[test]
fn noops_when_truncating_to_larger_len() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.truncate(4);

    assert_eq!(bits.bit_len(), 3);
    assert_eq!(bits.to_string(), "101");
}
