use crate::BitString;

// ---------------------------------------------------------------------------
// contains
// ---------------------------------------------------------------------------

#[test]
fn contains_basic() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr();
    let bs = BitString::try_from("01").unwrap();
    assert!(v.contains(&bs.as_bitstr()));
    let bs = BitString::try_from("111").unwrap();
    assert!(!v.contains(&bs.as_bitstr()));
}

#[test]
fn contains_empty_needle() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();
    let empty = BitString::new();
    assert!(v.contains(&empty.as_bitstr()));
}

#[test]
fn contains_longer_than_view_returns_false() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bitstr();
    let bs = BitString::try_from("101").unwrap();
    assert!(!v.contains(&bs.as_bitstr()));
}

#[test]
fn contains_on_offset_view() {
    let bits = BitString::try_from("110010").unwrap();
    let v = bits.as_bitstr().slice_from(1).slice_until(5);
    let bs = BitString::try_from("001").unwrap();
    assert!(v.contains(&bs.as_bitstr()));
    let bs = BitString::try_from("11").unwrap();
    assert!(!v.contains(&bs.as_bitstr()));
}

// ---------------------------------------------------------------------------
// find
// ---------------------------------------------------------------------------

#[test]
fn find_first_occurrence() {
    let bits = BitString::try_from("10110010").unwrap();
    let v = bits.as_bitstr();
    let bs = BitString::try_from("10").unwrap();
    assert_eq!(v.find(&bs.as_bitstr()), Some(0));
    let bs = BitString::try_from("01").unwrap();
    assert_eq!(v.find(&bs.as_bitstr()), Some(1));
    let bs = BitString::try_from("111").unwrap();
    assert_eq!(v.find(&bs.as_bitstr()), None);
}

#[test]
fn find_empty_needle_returns_zero() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();
    let empty = BitString::new();
    assert_eq!(v.find(&empty.as_bitstr()), Some(0));
}

#[test]
fn find_longer_than_view_returns_none() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bitstr();
    let bs = BitString::try_from("101").unwrap();
    assert_eq!(v.find(&bs.as_bitstr()), None);
}

#[test]
fn find_on_offset_view() {
    let bits = BitString::try_from("11001001").unwrap();
    let v = bits.as_bitstr().slice_from(2).slice_until(7);
    let bs = BitString::try_from("10").unwrap();
    assert_eq!(v.find(&bs.as_bitstr()), Some(2));
    let bs = BitString::try_from("00").unwrap();
    assert_eq!(v.find(&bs.as_bitstr()), Some(0));
}

#[test]
fn find_at_end_of_view() {
    let bits = BitString::try_from("11100").unwrap();
    let v = bits.as_bitstr();
    let bs = BitString::try_from("00").unwrap();
    assert_eq!(v.find(&bs.as_bitstr()), Some(3));
}

// ---------------------------------------------------------------------------
// rfind
// ---------------------------------------------------------------------------

#[test]
fn rfind_last_occurrence() {
    let bits = BitString::try_from("10110010").unwrap();
    let v = bits.as_bitstr();
    let bs = BitString::try_from("10").unwrap();
    assert_eq!(v.rfind(&bs.as_bitstr()), Some(6));
}

#[test]
fn rfind_empty_needle_returns_bit_len() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();
    let empty = BitString::new();
    assert_eq!(v.rfind(&empty.as_bitstr()), Some(5));
}

#[test]
fn rfind_longer_than_view_returns_none() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bitstr();
    let bs = BitString::try_from("101").unwrap();
    assert_eq!(v.rfind(&bs.as_bitstr()), None);
}

#[test]
fn rfind_on_offset_view() {
    let bits = BitString::try_from("11001001").unwrap();
    let v = bits.as_bitstr().slice_from(2).slice_until(7);
    let bs = BitString::try_from("00").unwrap();
    assert_eq!(v.rfind(&bs.as_bitstr()), Some(3));
}

#[test]
fn find_and_rfind_needle_appears_once() {
    let bits = BitString::try_from("11010").unwrap();
    let v = bits.as_bitstr();
    let bs = BitString::try_from("101").unwrap();
    assert_eq!(v.find(&bs.as_bitstr()), Some(1));
    assert_eq!(v.rfind(&bs.as_bitstr()), Some(1));
}
