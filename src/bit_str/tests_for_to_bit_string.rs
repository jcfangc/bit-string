use alloc::string::ToString;

use crate::BitString;

#[test]
fn full_view_roundtrip() {
    let bits = BitString::try_from("101001").unwrap();
    let view = bits.as_bit_str();
    let owned = view.to_bit_string();
    assert_eq!(owned, bits);
}

#[test]
fn offset_view_roundtrip() {
    let bits = BitString::try_from("110010").unwrap();
    let view = bits.as_bit_str().slice_from(2).slice_until(5);
    let owned = view.to_bit_string();
    assert_eq!(owned.to_string(), "0010");
    assert_eq!(owned.bit_len(), 4);
}

#[test]
fn empty_view_to_empty_bit_string() {
    let bits = BitString::try_from("101").unwrap();
    let view = bits.as_bit_str().slice_from(0).slice_until(0);
    let owned = view.to_bit_string();
    assert!(owned.is_empty());
}

#[test]
fn empty_source_to_empty_bit_string() {
    let bits = BitString::new();
    let view = bits.as_bit_str();
    let owned = view.to_bit_string();
    assert!(owned.is_empty());
}

#[test]
fn across_word_boundary() {
    let mut bits = BitString::zeros(130);
    bits.set(62, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);

    let view = bits.as_bit_str().slice_from(60).slice_until(70);
    let owned = view.to_bit_string();
    assert_eq!(view, owned.as_bit_str());
}
