use crate::BitString;
use alloc::string::ToString;

#[test]
fn removes_first_bit() {
    let mut bits = BitString::try_from("10101").unwrap();

    let removed = bits.remove(0);

    assert!(removed);
    assert_eq!(bits.bit_len(), 4);
    assert_eq!(bits.to_string(), "0101");
}

#[test]
fn removes_middle_bit() {
    let mut bits = BitString::try_from("10001").unwrap();

    let removed = bits.remove(2);

    assert!(!removed);
    assert_eq!(bits.bit_len(), 4);
    assert_eq!(bits.to_string(), "1001");
}

#[test]
fn removes_last_bit() {
    let mut bits = BitString::try_from("10101").unwrap();

    let removed = bits.remove(4);

    assert!(removed);
    assert_eq!(bits.bit_len(), 4);
    assert_eq!(bits.to_string(), "1010");
    assert_eq!(bits.get(4), None);
}

#[test]
fn removes_only_bit() {
    let mut bits = BitString::try_from("1").unwrap();

    let removed = bits.remove(0);

    assert!(removed);
    assert_eq!(bits.bit_len(), 0);
    assert!(bits.is_empty());
    assert_eq!(bits.to_string(), "");
}

#[test]
fn removes_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    bits.set(62, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let removed = bits.remove(63);

    assert!(removed);
    assert_eq!(bits.bit_len(), 129);

    assert_eq!(bits.get(62), Some(true));
    assert_eq!(bits.get(63), Some(true)); // old 64
    assert_eq!(bits.get(64), Some(true)); // old 65
    assert_eq!(bits.get(65), Some(false)); // old 66
    assert_eq!(bits.get(128), Some(true)); // old 129
    assert_eq!(bits.get(129), None);
}

#[test]
fn remove_preserves_counts_after_shift() {
    let mut bits = BitString::try_from("111001").unwrap();

    let removed = bits.remove(3);

    assert!(!removed);
    assert_eq!(bits.to_string(), "11101");
    assert_eq!(bits.count_ones(), 4);
    assert_eq!(bits.count_zeros(), 1);
}

#[test]
#[should_panic(expected = "bit string remove index out of bounds")]
fn panics_when_removing_at_len() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.remove(3);
}

#[test]
#[should_panic(expected = "bit string remove index out of bounds")]
fn panics_when_removing_from_empty() {
    let mut bits = BitString::new();

    bits.remove(0);
}
