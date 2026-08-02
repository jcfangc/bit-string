use super::*;
use crate::packed_string::tests_for_support::Letter;

#[test]
fn iter_is_double_ended_and_exact_size() {
    let value = PackedString::from_chars([Letter::A, Letter::B, Letter::C]);
    let mut iter = value.iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(Letter::A));
    assert_eq!(iter.next_back(), Some(Letter::C));
    assert_eq!(iter.next(), Some(Letter::B));
    assert_eq!(iter.next(), None);
}
