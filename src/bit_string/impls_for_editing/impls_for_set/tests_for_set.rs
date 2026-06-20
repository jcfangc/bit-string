use crate::BitString;
use alloc::string::ToString;

#[test]
fn returns_none_for_empty_bit_string() {
    let mut bits = BitString::new();

    assert_eq!(bits.set(0, true), None);
    assert_eq!(bits.to_string(), "");
}

#[test]
fn returns_none_for_out_of_bounds_index() {
    let mut bits = BitString::try_from("101").unwrap();

    assert_eq!(bits.set(3, true), None);
    assert_eq!(bits.set(usize::MAX, false), None);
    assert_eq!(bits.to_string(), "101");
}

#[test]
fn sets_false_to_true_and_returns_old_value() {
    let mut bits = BitString::try_from("1001").unwrap();

    assert_eq!(bits.set(1, true), Some(false));
    assert_eq!(bits.to_string(), "1101");
}

#[test]
fn sets_true_to_false_and_returns_old_value() {
    let mut bits = BitString::try_from("1001").unwrap();

    assert_eq!(bits.set(0, false), Some(true));
    assert_eq!(bits.to_string(), "0001");
}

#[test]
fn setting_to_same_value_preserves_bit_string() {
    let mut bits = BitString::try_from("1010").unwrap();

    assert_eq!(bits.set(0, true), Some(true));
    assert_eq!(bits.set(1, false), Some(false));
    assert_eq!(bits.to_string(), "1010");
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(130);

    assert_eq!(bits.set(0, true), Some(false));
    assert_eq!(bits.set(63, true), Some(false));
    assert_eq!(bits.set(64, true), Some(false));
    assert_eq!(bits.set(65, true), Some(false));
    assert_eq!(bits.set(129, true), Some(false));

    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(62), Some(false));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), Some(true));
    assert_eq!(bits.get(128), Some(false));
    assert_eq!(bits.get(129), Some(true));
    assert_eq!(bits.get(130), None);
}

#[test]
fn overwrites_existing_bits_across_word_boundaries() {
    let mut bits = BitString::ones(130);

    assert_eq!(bits.set(0, false), Some(true));
    assert_eq!(bits.set(63, false), Some(true));
    assert_eq!(bits.set(64, false), Some(true));
    assert_eq!(bits.set(65, false), Some(true));
    assert_eq!(bits.set(129, false), Some(true));

    assert_eq!(bits.get(0), Some(false));
    assert_eq!(bits.get(1), Some(true));
    assert_eq!(bits.get(63), Some(false));
    assert_eq!(bits.get(64), Some(false));
    assert_eq!(bits.get(65), Some(false));
    assert_eq!(bits.get(66), Some(true));
    assert_eq!(bits.get(129), Some(false));
}
