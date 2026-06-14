use alloc::string::ToString;

use crate::BitString;

#[test]
fn constructs_empty_from_empty_words() {
    let bits = BitString::from_words(&[], 0).unwrap();

    assert_eq!(bits.len(), 0);
    assert!(bits.is_empty());
    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.to_string(), "");
}

#[test]
fn constructs_single_partial_word() {
    let bits = BitString::from_words(&[0b1011], 4).unwrap();

    assert_eq!(bits.len(), 4);
    assert_eq!(bits.to_string(), "1101");
    assert_eq!(bits.count_ones(), 3);
}

#[test]
fn masks_unused_high_bits_in_last_word() {
    let bits = BitString::from_words(&[u64::MAX], 3).unwrap();

    assert_eq!(bits.len(), 3);
    assert_eq!(bits.to_string(), "111");
    assert_eq!(bits.count_ones(), 3);
    assert!(bits.is_all_ones());
}

#[test]
fn constructs_word_aligned_bits() {
    let bits = BitString::from_words(&[u64::MAX], 64).unwrap();

    assert_eq!(bits.len(), 64);
    assert_eq!(bits.count_ones(), 64);
    assert!(bits.is_all_ones());
}

#[test]
fn constructs_across_multiple_words() {
    let bits = BitString::from_words(&[1, 1], 65).unwrap();

    assert_eq!(bits.len(), 65);
    assert_eq!(bits.count_ones(), 2);
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(63), Some(false));
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), None);
}

#[test]
fn rejects_too_few_words() {
    assert!(BitString::from_words(&[], 1).is_none());
    assert!(BitString::from_words(&[0], 65).is_none());
}

#[test]
fn rejects_too_many_words() {
    assert!(BitString::from_words(&[0], 0).is_none());
    assert!(BitString::from_words(&[0, 0], 64).is_none());
    assert!(BitString::from_words(&[0, 0, 0], 65).is_none());
}
