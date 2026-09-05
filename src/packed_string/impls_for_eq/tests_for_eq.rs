use crate::packed_string::tests_for_support::{Letter, LetterString};

#[test]
fn equality_uses_length_and_packed_values() {
    let left = LetterString::from_chars([Letter::A, Letter::B]);
    let same = LetterString::from_chars([Letter::A, Letter::B]);
    let different = LetterString::from_chars([Letter::A, Letter::C]);
    assert_eq!(left, same);
    assert_ne!(left, different);
}
