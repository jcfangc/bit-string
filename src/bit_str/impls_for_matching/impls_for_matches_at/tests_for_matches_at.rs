use crate::BitString;

// ---------------------------------------------------------------------------
// matches_at
// ---------------------------------------------------------------------------

#[test]
fn matches_at_exact_match() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::try_from("10").unwrap();
    assert!(v.matches_at(0, p.as_bit_str()));
    assert!(!v.matches_at(1, p.as_bit_str()));
}

#[test]
fn matches_at_beyond_view_returns_false() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str().slice_from(2);
    let p = BitString::try_from("10").unwrap();
    assert!(!v.matches_at(2, p.as_bit_str()));
}

#[test]
fn matches_at_pattern_too_long_returns_false() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::try_from("100").unwrap();
    assert!(!v.matches_at(0, p.as_bit_str()));
}

#[test]
fn matches_at_empty_pattern_always_true() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::new();
    assert!(v.matches_at(0, p.as_bit_str()));
    assert!(v.matches_at(3, p.as_bit_str()));
}

#[test]
fn matches_at_on_offset_view() {
    let bits = BitString::try_from("11100011").unwrap();
    // bits: 1 1 1 0 0 0 1 1, view 2..7 → 1 0 0 0 1
    let v = bits.as_bit_str().slice_from(2).slice_until(7);
    let p = BitString::try_from("0001").unwrap();
    assert!(v.matches_at(1, p.as_bit_str()));
}
