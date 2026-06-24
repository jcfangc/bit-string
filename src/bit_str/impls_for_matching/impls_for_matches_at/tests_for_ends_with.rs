use crate::BitString;

// ---------------------------------------------------------------------------
// ends_with
// ---------------------------------------------------------------------------

#[test]
fn ends_with_basic() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bit_str();
    let s = BitString::try_from("100").unwrap();
    assert!(v.ends_with(s.as_bit_str()));
    let s = BitString::try_from("10").unwrap();
    assert!(!v.ends_with(s.as_bit_str()));
    let s = BitString::try_from("101100").unwrap();
    assert!(v.ends_with(s.as_bit_str()));
}

#[test]
fn ends_with_empty_suffix() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();
    let s = BitString::new();
    assert!(v.ends_with(s.as_bit_str()));
}

#[test]
fn ends_with_longer_suffix_returns_false() {
    let bits = BitString::try_from("101").unwrap();
    let v = bits.as_bit_str();
    let s = BitString::try_from("1101").unwrap();
    assert!(!v.ends_with(s.as_bit_str()));
}

#[test]
fn ends_with_on_offset_view() {
    let bits = BitString::try_from("110101").unwrap();
    // bits 1..6 → 1 0 1 0 1
    let v = bits.as_bit_str().slice_from(1).slice_until(5);
    let s = BitString::try_from("01").unwrap();
    assert!(v.ends_with(s.as_bit_str()));
    let s = BitString::try_from("10").unwrap();
    assert!(!v.ends_with(s.as_bit_str()));
}

#[test]
fn matches_and_ends_with_across_word_boundaries() {
    let mut bits = BitString::zeros(130);
    bits.set(62, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    let v = bits.as_bit_str();
    let p = BitString::try_from("1111").unwrap();
    assert!(v.matches_at(62, p.as_bit_str()));
}
