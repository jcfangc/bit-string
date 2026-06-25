use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::BitString;

// ---------------------------------------------------------------------------
// Basic equal / different
// ---------------------------------------------------------------------------

#[test]
fn equal_strings_are_equal() {
    let a = BitString::try_from("101001").unwrap();
    let b = BitString::try_from("101001").unwrap();
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Equal);
}

#[test]
fn different_at_first_bit() {
    // "0..." < "1..."
    let a = BitString::try_from("011").unwrap();
    let b = BitString::try_from("111").unwrap();
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Less);
    assert_eq!(b.as_bit_str().cmp(&a.as_bit_str()), Ordering::Greater);
}

#[test]
fn different_at_later_bit() {
    // "101" vs "100" — differ at bit 2
    let a = BitString::try_from("100").unwrap();
    let b = BitString::try_from("101").unwrap();
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Less);
    assert_eq!(b.as_bit_str().cmp(&a.as_bit_str()), Ordering::Greater);
}

// ---------------------------------------------------------------------------
// Prefix relationship — longer wins
// ---------------------------------------------------------------------------

#[test]
fn prefix_shorter_is_less() {
    let a = BitString::try_from("101").unwrap();
    let b = BitString::try_from("1010").unwrap();
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Less);
}

#[test]
fn longer_prefix_is_greater() {
    let a = BitString::try_from("1010").unwrap();
    let b = BitString::try_from("101").unwrap();
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Greater);
}

// ---------------------------------------------------------------------------
// Empty
// ---------------------------------------------------------------------------

#[test]
fn empty_is_less_than_nonempty() {
    let a = BitString::new();
    let b = BitString::try_from("0").unwrap();
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Less);
}

#[test]
fn empty_equals_empty() {
    let a = BitString::new();
    let b = BitString::new();
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Equal);
}

// ---------------------------------------------------------------------------
// Cross-word boundary (aligned fast path)
// ---------------------------------------------------------------------------

#[test]
fn cross_word_boundary_same_length() {
    // 128 bits each, differ in the second word
    let mut a = BitString::zeros(128);
    let mut b = BitString::zeros(128);
    a.set(70, true); // second word, bit 6
    b.set(70, false);
    b.set(71, true); // second word, bit 7

    // a[0..70] == b[0..70] (all zeros), a[70]=1, b[70]=0 → a > b
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Greater);
    assert_eq!(b.as_bit_str().cmp(&a.as_bit_str()), Ordering::Less);
}

#[test]
fn cross_word_boundary_different_in_first_word() {
    let mut a = BitString::zeros(130);
    let b = BitString::zeros(130);
    a.set(3, true);
    // a > b (a[3]=1, b[3]=0, all others zero)
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Greater);
}

// ---------------------------------------------------------------------------
// Offset views (unaligned)
// ---------------------------------------------------------------------------

#[test]
fn unaligned_views_compare_correctly() {
    let source = BitString::try_from("111111111100000000001111111111").unwrap();
    // view starting at bit 3: "111111100000000001111111111"
    // view starting at bit 4: "11111100000000001111111111"
    let v1 = source.as_bit_str().slice_from(3);
    let v2 = source.as_bit_str().slice_from(4);
    // v1 > v2 because at common offset, v1[0]=1, v2[0]=1 (equal at bit 0
    // of the views), and eventually v1 is longer
    // Actually v1 is one bit longer
    assert_eq!(v1.cmp(&v2), Ordering::Greater);
}

#[test]
fn unaligned_vs_aligned_same_content() {
    let source = BitString::zeros(200);
    let v = source.as_bit_str().slice_from(3).slice_until(197);
    let owned = v.to_bit_string();
    assert_eq!(v.cmp(&owned.as_bit_str()), Ordering::Equal);
}

// ---------------------------------------------------------------------------
// PartialOrd / Ord trait usage
// ---------------------------------------------------------------------------

#[test]
fn partial_cmp_is_some() {
    let a = BitString::try_from("101").unwrap();
    let b = BitString::try_from("100").unwrap();
    assert_eq!(
        a.as_bit_str().partial_cmp(&b.as_bit_str()),
        Some(Ordering::Greater)
    );
}

#[test]
fn sort_bit_strs() {
    let mut strings: Vec<BitString> = ["101", "001", "111", "000", "010"]
        .into_iter()
        .map(|s| BitString::try_from(s).unwrap())
        .collect();
    strings.sort_by(|a, b| a.as_bit_str().cmp(&b.as_bit_str()));

    let sorted: Vec<String> = strings.iter().map(|bs| bs.to_string()).collect();
    assert_eq!(sorted, vec!["000", "001", "010", "101", "111"]);
}

#[test]
fn min_max() {
    let a = BitString::try_from("0011").unwrap();
    let b = BitString::try_from("1100").unwrap();
    let min = a.as_bit_str().min(b.as_bit_str());
    let max = a.as_bit_str().max(b.as_bit_str());
    assert_eq!(min, a.as_bit_str());
    assert_eq!(max, b.as_bit_str());
}

// ---------------------------------------------------------------------------
// Transitivity / consistency
// ---------------------------------------------------------------------------

#[test]
fn eq_implies_not_less_and_not_greater() {
    let a = BitString::try_from("101010").unwrap();
    let b = BitString::try_from("101010").unwrap();
    let va = a.as_bit_str();
    let vb = b.as_bit_str();
    assert_eq!(va, vb);
    assert!(!(va < vb));
    assert!(!(va > vb));
    assert!(va <= vb);
    assert!(va >= vb);
}

#[test]
fn ordering_consistent_with_equality() {
    for bits in ["", "0", "1", "10", "01", "111", "000", "101010"] {
        let bs = BitString::try_from(bits).unwrap();
        let v = bs.as_bit_str();
        assert_eq!(v.cmp(&v), Ordering::Equal, "self-cmp failed for {bits:?}");
    }
}

// ---------------------------------------------------------------------------
// Large input — SIMD paths
// ---------------------------------------------------------------------------

#[test]
fn large_identical() {
    let a = BitString::zeros(65536);
    let b = BitString::zeros(65536);
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Equal);
}

#[test]
fn large_differ_at_last_bit() {
    let a = BitString::zeros(65536);
    let mut b = BitString::zeros(65536);
    b.set(65535, true);
    // a < b (a[65535]=0, b[65535]=1)
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Less);
}

#[test]
fn large_differ_at_first_bit() {
    let a = BitString::zeros(65536);
    let mut b = BitString::zeros(65536);
    b.set(0, true);
    assert_eq!(a.as_bit_str().cmp(&b.as_bit_str()), Ordering::Less);
}
