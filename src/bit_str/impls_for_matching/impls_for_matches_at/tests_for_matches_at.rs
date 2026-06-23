use crate::BitString;

// ---------------------------------------------------------------------------
// matches_at
// ---------------------------------------------------------------------------

#[test]
fn matches_at_exact_match() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr();
    let p = BitString::try_from("10").unwrap();
    assert!(v.matches_at(0, &p.as_bitstr()));
    assert!(!v.matches_at(1, &p.as_bitstr()));
}

#[test]
fn matches_at_beyond_view_returns_false() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr().slice_from(2);
    let p = BitString::try_from("10").unwrap();
    assert!(!v.matches_at(2, &p.as_bitstr()));
}

#[test]
fn matches_at_pattern_too_long_returns_false() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bitstr();
    let p = BitString::try_from("100").unwrap();
    assert!(!v.matches_at(0, &p.as_bitstr()));
}

#[test]
fn matches_at_empty_pattern_always_true() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();
    let p = BitString::new();
    assert!(v.matches_at(0, &p.as_bitstr()));
    assert!(v.matches_at(3, &p.as_bitstr()));
}

#[test]
fn matches_at_on_offset_view() {
    let bits = BitString::try_from("11100011").unwrap();
    // bits: 1 1 1 0 0 0 1 1, view 2..7 → 1 0 0 0 1
    let v = bits.as_bitstr().slice_from(2).slice_until(7);
    let p = BitString::try_from("0001").unwrap();
    assert!(v.matches_at(1, &p.as_bitstr()));
}

// ---------------------------------------------------------------------------
// starts_with
// ---------------------------------------------------------------------------

#[test]
fn starts_with_basic() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr();
    let p = BitString::try_from("101").unwrap();
    assert!(v.starts_with(&p.as_bitstr()));
    let p = BitString::try_from("11").unwrap();
    assert!(!v.starts_with(&p.as_bitstr()));
    let p = BitString::try_from("101100").unwrap();
    assert!(v.starts_with(&p.as_bitstr()));
}

#[test]
fn starts_with_empty_prefix() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();
    let p = BitString::new();
    assert!(v.starts_with(&p.as_bitstr()));
}

#[test]
fn starts_with_longer_prefix_returns_false() {
    let bits = BitString::try_from("101").unwrap();
    let v = bits.as_bitstr();
    let p = BitString::try_from("1011").unwrap();
    assert!(!v.starts_with(&p.as_bitstr()));
}

#[test]
fn starts_with_on_offset_view() {
    let bits = BitString::try_from("110101").unwrap();
    let v = bits.as_bitstr().slice_from(1).slice_until(5);
    let p = BitString::try_from("10").unwrap();
    assert!(v.starts_with(&p.as_bitstr()));
    let p = BitString::try_from("11").unwrap();
    assert!(!v.starts_with(&p.as_bitstr()));
}

// ---------------------------------------------------------------------------
// ends_with
// ---------------------------------------------------------------------------

#[test]
fn ends_with_basic() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr();
    let s = BitString::try_from("100").unwrap();
    assert!(v.ends_with(&s.as_bitstr()));
    let s = BitString::try_from("10").unwrap();
    assert!(!v.ends_with(&s.as_bitstr()));
    let s = BitString::try_from("101100").unwrap();
    assert!(v.ends_with(&s.as_bitstr()));
}

#[test]
fn ends_with_empty_suffix() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();
    let s = BitString::new();
    assert!(v.ends_with(&s.as_bitstr()));
}

#[test]
fn ends_with_longer_suffix_returns_false() {
    let bits = BitString::try_from("101").unwrap();
    let v = bits.as_bitstr();
    let s = BitString::try_from("1101").unwrap();
    assert!(!v.ends_with(&s.as_bitstr()));
}

#[test]
fn ends_with_on_offset_view() {
    let bits = BitString::try_from("110101").unwrap();
    // bits 1..6 → 1 0 1 0 1
    let v = bits.as_bitstr().slice_from(1).slice_until(5);
    let s = BitString::try_from("01").unwrap();
    assert!(v.ends_with(&s.as_bitstr()));
    let s = BitString::try_from("10").unwrap();
    assert!(!v.ends_with(&s.as_bitstr()));
}

#[test]
fn matches_and_ends_with_across_word_boundaries() {
    let mut bits = BitString::zeros(130);
    bits.set(62, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    let v = bits.as_bitstr();
    let p = BitString::try_from("1111").unwrap();
    assert!(v.matches_at(62, &p.as_bitstr()));
}
