use crate::BitString;

#[test]
fn returns_false_when_suffix_is_longer() {
    let bits = BitString::try_from("101").unwrap();
    let suffix = BitString::try_from("0101").unwrap();

    assert!(!bits.ends_with(suffix.as_bit_str()));
}

#[test]
fn empty_suffix_always_matches() {
    let bits = BitString::try_from("101").unwrap();
    let suffix = BitString::new();

    assert!(bits.ends_with(suffix.as_bit_str()));
}

#[test]
fn matches_suffix() {
    let bits = BitString::try_from("101001").unwrap();
    let suffix = BitString::try_from("001").unwrap();

    assert!(bits.ends_with(suffix.as_bit_str()));
}

#[test]
fn rejects_non_suffix() {
    let bits = BitString::try_from("101001").unwrap();
    let suffix = BitString::try_from("101").unwrap();

    assert!(!bits.ends_with(suffix.as_bit_str()));
}

#[test]
fn equal_bit_strings_match_as_suffix() {
    let bits = BitString::try_from("101001").unwrap();
    let suffix = BitString::try_from("101001").unwrap();

    assert!(bits.ends_with(suffix.as_bit_str()));
}
