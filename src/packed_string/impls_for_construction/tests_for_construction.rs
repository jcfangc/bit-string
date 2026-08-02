use crate::packed_string::tests_for_support::{Letter, LetterString};

#[test]
fn enum_discriminants_are_stored_directly() {
    let value = LetterString::from_chars([Letter::A, Letter::B, Letter::C, Letter::D]);
    assert_eq!(value.bits_per_char(), 2);
    assert_eq!(value.bits().bit_len(), 8);
    assert_eq!(value.bits().get_chunk(0), 0b11_10_01_00);
}

#[test]
fn collect_constructs_a_packed_string() {
    let value: LetterString = [Letter::A, Letter::C].into_iter().collect();
    assert_eq!(value.char_len(), 2);
    assert_eq!(value.get(0), Some(Letter::A));
}
