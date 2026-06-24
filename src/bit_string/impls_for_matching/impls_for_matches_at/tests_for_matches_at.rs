use crate::BitString;

#[test]
fn returns_false_when_index_is_past_len() {
    let bits = BitString::try_from("1010").unwrap();
    let pattern = BitString::try_from("1").unwrap();

    assert!(!bits.matches_at(bits.bit_len() + 1, pattern.as_bit_str()));
}

#[test]
fn returns_false_when_pattern_does_not_fit_at_index() {
    let bits = BitString::try_from("1010").unwrap();
    let pattern = BitString::try_from("10").unwrap();

    assert!(!bits.matches_at(3, pattern.as_bit_str()));
    assert!(!bits.matches_at(bits.bit_len(), pattern.as_bit_str()));
}

#[test]
fn allows_empty_pattern_at_len() {
    let bits = BitString::try_from("1010").unwrap();
    let pattern = BitString::new();

    assert!(bits.matches_at(bits.bit_len(), pattern.as_bit_str()));
}
