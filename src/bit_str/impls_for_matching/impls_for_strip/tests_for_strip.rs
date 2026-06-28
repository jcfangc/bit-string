use crate::BitString;

// ---------------------------------------------------------------------------
// strip_prefix
// ---------------------------------------------------------------------------

#[test]
fn strip_prefix_removes_matching_prefix() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::try_from("101").unwrap();
    let rest = v.strip_prefix_str(p.as_bit_str()).unwrap();
    assert_eq!(rest.bit_len(), 3);
    assert_eq!(rest.get(0), Some(true));
    assert_eq!(rest.get(1), Some(false));
    assert_eq!(rest.get(2), Some(false));
}

#[test]
fn strip_prefix_non_matching_returns_none() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::try_from("11").unwrap();
    assert!(v.strip_prefix_str(p.as_bit_str()).is_none());
}

#[test]
fn strip_prefix_empty() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::new();
    let rest = v.strip_prefix_str(p.as_bit_str()).unwrap();
    assert_eq!(rest.bit_len(), v.bit_len());
}

#[test]
fn strip_prefix_entire_view() {
    let bits = BitString::try_from("101").unwrap();
    let v = bits.as_bit_str();
    let p = BitString::try_from("101").unwrap();
    let rest = v.strip_prefix_str(p.as_bit_str()).unwrap();
    assert_eq!(rest.bit_len(), 0);
}

#[test]
fn strip_prefix_on_offset_view() {
    let bits = BitString::try_from("110101").unwrap();
    // view bits 1..6 → 1 0 1 0 1
    let v = bits.as_bit_str().slice_from(1).slice_until(5);
    let p = BitString::try_from("10").unwrap();
    let rest = v.strip_prefix_str(p.as_bit_str()).unwrap();
    assert_eq!(rest.bit_len(), 3);
    assert_eq!(rest.get(0), Some(true));
    assert_eq!(rest.get(1), Some(false));
    assert_eq!(rest.get(2), Some(true));
}

// ---------------------------------------------------------------------------
// strip_suffix
// ---------------------------------------------------------------------------

#[test]
fn strip_suffix_removes_matching_suffix() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bit_str();
    let s = BitString::try_from("100").unwrap();
    let rest = v.strip_suffix_str(s.as_bit_str()).unwrap();
    assert_eq!(rest.bit_len(), 3);
    assert_eq!(rest.get(0), Some(true));
    assert_eq!(rest.get(1), Some(false));
    assert_eq!(rest.get(2), Some(true));
}

#[test]
fn strip_suffix_non_matching_returns_none() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bit_str();
    let s = BitString::try_from("10").unwrap();
    assert!(v.strip_suffix_str(s.as_bit_str()).is_none());
}

#[test]
fn strip_suffix_empty() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bit_str();
    let s = BitString::new();
    let rest = v.strip_suffix_str(s.as_bit_str()).unwrap();
    assert_eq!(rest.bit_len(), v.bit_len());
}

#[test]
fn strip_suffix_entire_view() {
    let bits = BitString::try_from("101").unwrap();
    let v = bits.as_bit_str();
    let s = BitString::try_from("101").unwrap();
    let rest = v.strip_suffix_str(s.as_bit_str()).unwrap();
    assert_eq!(rest.bit_len(), 0);
}

#[test]
fn strip_suffix_on_offset_view() {
    let bits = BitString::try_from("110101").unwrap();
    // view bits 1..6 → 1 0 1 0 1
    let v = bits.as_bit_str().slice_from(1).slice_until(5);
    let s = BitString::try_from("01").unwrap();
    let rest = v.strip_suffix_str(s.as_bit_str()).unwrap();
    assert_eq!(rest.bit_len(), 3);
    assert_eq!(rest.get(0), Some(true));
    assert_eq!(rest.get(1), Some(false));
    assert_eq!(rest.get(2), Some(true));
}
