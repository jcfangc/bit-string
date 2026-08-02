use super::*;
use crate::packed_string::tests_for_support::Letter;

#[test]
fn push_set_pop_and_truncate_preserve_values() {
    let mut value = PackedString::from_chars([Letter::A, Letter::B, Letter::C]);
    value.push(Letter::D);
    assert_eq!(value.set(1, Letter::C), Some(Letter::B));
    assert_eq!(value.pop(), Some(Letter::D));
    value.truncate(2);
    assert_eq!(
        value.iter().collect::<alloc::vec::Vec<_>>(),
        [Letter::A, Letter::C]
    );
    value.clear();
    assert!(value.is_empty());
}
