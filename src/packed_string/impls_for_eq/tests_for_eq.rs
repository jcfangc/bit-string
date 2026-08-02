use super::*;
use crate::packed_string::tests_for_support::Letter;

#[test]
fn equality_uses_length_and_packed_values() {
    let left = PackedString::from_chars([Letter::A, Letter::B]);
    let same = PackedString::from_chars([Letter::A, Letter::B]);
    let different = PackedString::from_chars([Letter::A, Letter::C]);
    assert_eq!(left, same);
    assert_ne!(left, different);
}
