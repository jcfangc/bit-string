use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::BitString;

// ---------------------------------------------------------------------------
// BitString Ord — delegates to as_bit_str().cmp()
// ---------------------------------------------------------------------------

#[test]
fn equal_bit_strings_are_ordered_equal() {
    let a = BitString::try_from("101001").unwrap();
    let b = BitString::try_from("101001").unwrap();
    assert_eq!(a, b);
    assert_eq!(a.cmp(&b), Ordering::Equal);
}

#[test]
fn less_greater() {
    let a = BitString::try_from("100").unwrap();
    let b = BitString::try_from("101").unwrap();
    assert!(a < b);
    assert!(b > a);
}

#[test]
fn prefix_shorter_is_less() {
    let a = BitString::try_from("101").unwrap();
    let b = BitString::try_from("1010").unwrap();
    assert!(a < b);
    assert!(b > a);
}

#[test]
fn empty_is_less_than_nonempty() {
    let a = BitString::new();
    let b = BitString::try_from("0").unwrap();
    assert!(a < b);
    assert!(a <= b);
}

#[test]
fn empty_equals_empty() {
    let a = BitString::new();
    let b = BitString::new();
    assert_eq!(a.cmp(&b), Ordering::Equal);
}

#[test]
fn sort_bit_strings() {
    let mut strings: Vec<BitString> = ["101", "001", "111", "000", "010"]
        .into_iter()
        .map(|s| BitString::try_from(s).unwrap())
        .collect();
    strings.sort();

    let sorted: Vec<String> = strings.iter().map(|bs| bs.to_string()).collect();
    assert_eq!(sorted, vec!["000", "001", "010", "101", "111"]);
}

#[test]
fn min_max() {
    let a = BitString::try_from("0011").unwrap();
    let b = BitString::try_from("1100").unwrap();
    let a2 = a.clone();
    let b2 = b.clone();
    let b3 = b.clone();
    assert_eq!(a.min(b), a2);
    assert_eq!(a2.max(b2), b3);
}

#[test]
fn cmp_consistent_with_bit_str() {
    for bits in ["", "0", "1", "10", "01", "111", "000", "101010", "1100"] {
        let x = BitString::try_from(bits).unwrap();
        let y = BitString::try_from(bits).unwrap();
        assert_eq!(
            x.cmp(&y),
            x.as_bit_str().cmp(&y.as_bit_str()),
            "mismatch for {bits:?}"
        );
    }
}

#[test]
fn cross_word_different() {
    let mut a = BitString::zeros(128);
    let mut b = BitString::zeros(128);
    a.set(70, true);
    b.set(71, true);
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), a.cmp(&b));
}

// ---------------------------------------------------------------------------
// BitString PartialOrd — delegates to cmp()
// ---------------------------------------------------------------------------

#[test]
fn partial_cmp_is_some() {
    let a = BitString::try_from("101").unwrap();
    let b = BitString::try_from("100").unwrap();
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
}
