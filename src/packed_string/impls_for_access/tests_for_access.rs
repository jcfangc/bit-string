use crate::packed_string::tests_for_support::{Letter, LetterString, Only, OnlyString};

#[test]
fn get_first_and_last_decode_values() {
    let value = LetterString::from_chars([Letter::A, Letter::B, Letter::C]);
    assert_eq!(value.first(), Some(Letter::A));
    assert_eq!(value.last(), Some(Letter::C));
    assert_eq!(value.get(3), None);
}

#[test]
fn one_value_type_needs_no_payload_bits() {
    let value = OnlyString::from_chars([Only::Value; 100]);
    assert_eq!(value.char_len(), 100);
    assert_eq!(value.bits().bit_len(), 0);
    assert_eq!(value.get(99), Some(Only::Value));
}
