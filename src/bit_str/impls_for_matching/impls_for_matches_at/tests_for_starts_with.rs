use crate::BitString;

// ---------------------------------------------------------------------------
// starts_with
// ---------------------------------------------------------------------------

#[test]
fn starts_with_basic() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::try_from("101").unwrap();
    assert!(v.starts_with_str(p.as_bit_str()));
    let p = BitString::try_from("11").unwrap();
    assert!(!v.starts_with_str(p.as_bit_str()));
    let p = BitString::try_from("101100").unwrap();
    assert!(v.starts_with_str(p.as_bit_str()));
}

#[test]
fn starts_with_empty_prefix() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::new();
    assert!(v.starts_with_str(p.as_bit_str()));
}

#[test]
fn starts_with_longer_prefix_returns_false() {
    let bits = BitString::try_from("101").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::try_from("1011").unwrap();
    assert!(!v.starts_with_str(p.as_bit_str()));
}

#[test]
fn starts_with_on_offset_view() {
    let bits = BitString::try_from("110101").unwrap();
    let v = bits.as_bit_str().slice_from(1).slice_until(5);
    let p = BitString::try_from("10").unwrap();
    assert!(v.starts_with_str(p.as_bit_str()));
    let p = BitString::try_from("11").unwrap();
    assert!(!v.starts_with_str(p.as_bit_str()));
}
