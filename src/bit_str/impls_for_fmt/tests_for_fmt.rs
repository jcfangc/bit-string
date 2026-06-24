use alloc::string::ToString;

use crate::BitString;

#[test]
fn display_full_view() {
    let bits = BitString::try_from("101001").unwrap();
    assert_eq!(bits.as_bit_str().to_string(), "101001");
}

#[test]
fn display_empty_view() {
    let bits = BitString::new();
    assert_eq!(bits.as_bit_str().to_string(), "");
}

#[test]
fn display_offset_view() {
    let bits = BitString::try_from("110010").unwrap();
    let v = bits.as_bit_str().slice_from(2).slice_until(5);
    assert_eq!(v.to_string(), "0010");
}

#[test]
fn display_single_bit_views() {
    let bits = BitString::try_from("10").unwrap();
    assert_eq!(
        bits.as_bit_str().slice_from(0).slice_until(1).to_string(),
        "1"
    );
    assert_eq!(
        bits.as_bit_str().slice_from(1).slice_until(2).to_string(),
        "0"
    );
}

#[test]
fn display_across_word_boundary() {
    let mut bits = BitString::zeros(130);
    bits.set(62, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    let expected = bits.to_string();
    assert_eq!(bits.as_bit_str().to_string(), expected);
}

#[test]
fn debug_format() {
    let bits = BitString::try_from("1010").unwrap();
    let v = bits.as_bit_str();
    assert_eq!(alloc::format!("{v:?}"), "BitStr(\"1010\")");

    let v = bits.as_bit_str().slice_from(1).slice_until(3);
    assert_eq!(alloc::format!("{v:?}"), "BitStr(\"010\")");
}

#[test]
fn debug_empty_view() {
    let bits = BitString::new();
    let v = bits.as_bit_str();
    assert_eq!(alloc::format!("{v:?}"), "BitStr(\"\")");
}
