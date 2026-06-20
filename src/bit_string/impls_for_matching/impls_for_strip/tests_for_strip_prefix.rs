use alloc::string::ToString;

use crate::BitString;

#[test]
fn empty_prefix_returns_original_copy() {
    let bits = BitString::try_from("101001").unwrap();
    let prefix = BitString::new();

    let stripped = bits.strip_prefix(&prefix).unwrap();

    assert_eq!(stripped.to_string(), "101001");
}

#[test]
fn strips_matching_prefix() {
    let bits = BitString::try_from("101001").unwrap();
    let prefix = BitString::try_from("101").unwrap();

    let stripped = bits.strip_prefix(&prefix).unwrap();

    assert_eq!(stripped.to_string(), "001");
}

#[test]
fn equal_prefix_returns_empty_bit_string() {
    let bits = BitString::try_from("101001").unwrap();
    let prefix = BitString::try_from("101001").unwrap();

    let stripped = bits.strip_prefix(&prefix).unwrap();

    assert!(stripped.is_empty());
    assert_eq!(stripped.to_string(), "");
}

#[test]
fn returns_none_when_prefix_does_not_match() {
    let bits = BitString::try_from("101001").unwrap();
    let prefix = BitString::try_from("100").unwrap();

    assert_eq!(bits.strip_prefix(&prefix), None);
}

#[test]
fn returns_none_when_prefix_is_longer() {
    let bits = BitString::try_from("101").unwrap();
    let prefix = BitString::try_from("1010").unwrap();

    assert_eq!(bits.strip_prefix(&prefix), None);
}
