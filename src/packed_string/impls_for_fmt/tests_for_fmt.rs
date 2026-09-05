use crate::packed_string::tests_for_support::{Letter, LetterString};
use alloc::string::ToString;

#[test]
fn display_uses_the_character_display_implementation() {
    let value = LetterString::from_chars([Letter::A, Letter::B, Letter::C, Letter::D]);
    assert_eq!(value.to_string(), "ABCD");
}

#[test]
fn debug_lists_the_enum_values() {
    let value = LetterString::from_chars([Letter::A, Letter::B]);
    assert_eq!(alloc::format!("{value:?}"), "[A, B]");
}
