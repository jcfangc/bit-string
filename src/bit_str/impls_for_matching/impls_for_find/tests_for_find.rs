use crate::BitString;

// ---------------------------------------------------------------------------
// contains
// ---------------------------------------------------------------------------

#[test]
fn contains_basic() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr();

    assert!(v.contains(&BitString::try_from("01").unwrap()));
    assert!(!v.contains(&BitString::try_from("111").unwrap()));
}

#[test]
fn contains_empty_needle() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();
    assert!(v.contains(&BitString::new()));
}

#[test]
fn contains_longer_than_view_returns_false() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bitstr();
    assert!(!v.contains(&BitString::try_from("101").unwrap()));
}

#[test]
fn contains_on_offset_view() {
    let bits = BitString::try_from("110010").unwrap();
    // bits: 1 1 0 0 1 0
    // view bits 1..5 → 1 0 0 1
    let v = bits.as_bitstr().slice_from(1).slice_until(5);

    assert!(v.contains(&BitString::try_from("001").unwrap()));
    assert!(!v.contains(&BitString::try_from("11").unwrap()));
}

// ---------------------------------------------------------------------------
// find
// ---------------------------------------------------------------------------

#[test]
fn find_first_occurrence() {
    let bits = BitString::try_from("10110010").unwrap();
    let v = bits.as_bitstr();

    assert_eq!(v.find(&BitString::try_from("10").unwrap()), Some(0));
    assert_eq!(v.find(&BitString::try_from("01").unwrap()), Some(1));
    assert_eq!(v.find(&BitString::try_from("111").unwrap()), None);
}

#[test]
fn find_empty_needle_returns_zero() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();
    assert_eq!(v.find(&BitString::new()), Some(0));
}

#[test]
fn find_longer_than_view_returns_none() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bitstr();
    assert_eq!(v.find(&BitString::try_from("101").unwrap()), None);
}

#[test]
fn find_on_offset_view() {
    let bits = BitString::try_from("11001001").unwrap();
    // bits: 1 1 0 0 1 0 0 1
    // slice_from(2).slice_until(7) → bits 2..8 → 0 0 1 0 0 1 (len 6)
    let v = bits.as_bitstr().slice_from(2).slice_until(7);

    // First "10" is at view position 2 (original bits 4-5)
    assert_eq!(v.find(&BitString::try_from("10").unwrap()), Some(2));
    // "00" at view position 3 (original bits 5-6)
    assert_eq!(v.find(&BitString::try_from("00").unwrap()), Some(0));
}

#[test]
fn find_at_end_of_view() {
    let bits = BitString::try_from("11100").unwrap();
    let v = bits.as_bitstr();

    assert_eq!(v.find(&BitString::try_from("00").unwrap()), Some(3));
}

// ---------------------------------------------------------------------------
// rfind
// ---------------------------------------------------------------------------

#[test]
fn rfind_last_occurrence() {
    let bits = BitString::try_from("10110010").unwrap();
    let v = bits.as_bitstr();

    // "10" at positions 0, 5, 6 → last is 6
    assert_eq!(v.rfind(&BitString::try_from("10").unwrap()), Some(6));
}

#[test]
fn rfind_empty_needle_returns_bit_len() {
    let bits = BitString::try_from("10110").unwrap();
    let v = bits.as_bitstr();
    assert_eq!(v.rfind(&BitString::new()), Some(5));
}

#[test]
fn rfind_longer_than_view_returns_none() {
    let bits = BitString::try_from("10").unwrap();
    let v = bits.as_bitstr();
    assert_eq!(v.rfind(&BitString::try_from("101").unwrap()), None);
}

#[test]
fn rfind_on_offset_view() {
    let bits = BitString::try_from("11001001").unwrap();
    // view bits 2..7 → 0 0 1 0 0
    let v = bits.as_bitstr().slice_from(2).slice_until(7);

    // Last "00" is at view position 3
    assert_eq!(v.rfind(&BitString::try_from("00").unwrap()), Some(3));
}

#[test]
fn find_and_rfind_needle_appears_once() {
    let bits = BitString::try_from("11010").unwrap();
    let v = bits.as_bitstr();

    let f = v.find(&BitString::try_from("101").unwrap());
    let r = v.rfind(&BitString::try_from("101").unwrap());
    assert_eq!(f, r);
    assert_eq!(f, Some(1));
}
