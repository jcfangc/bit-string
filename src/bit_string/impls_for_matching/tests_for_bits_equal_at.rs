use crate::BitString;

use super::bits_equal_at;

#[test]
fn returns_true_when_needle_matches_at_offset() {
    let haystack = BitString::try_from("00110110").unwrap();
    let needle = BitString::try_from("110").unwrap();

    assert!(bits_equal_at(&haystack, 2, &needle));
    assert!(bits_equal_at(&haystack, 5, &needle));
}

#[test]
fn returns_false_when_needle_differs_at_offset() {
    let haystack = BitString::try_from("00110110").unwrap();
    let needle = BitString::try_from("110").unwrap();

    assert!(!bits_equal_at(&haystack, 0, &needle));
    assert!(!bits_equal_at(&haystack, 1, &needle));
    assert!(!bits_equal_at(&haystack, 3, &needle));
}

#[test]
fn empty_needle_matches_at_valid_boundary_offsets() {
    let haystack = BitString::try_from("101001").unwrap();
    let needle = BitString::new();

    assert!(bits_equal_at(&haystack, 0, &needle));
    assert!(bits_equal_at(&haystack, 3, &needle));
    assert!(bits_equal_at(&haystack, haystack.bit_len(), &needle));
}

#[test]
fn works_across_word_boundaries() {
    let mut haystack = BitString::zeros(130);

    haystack.set(63, true);
    haystack.set(64, true);
    haystack.set(65, true);

    let needle = BitString::try_from("01110").unwrap();

    assert!(bits_equal_at(&haystack, 62, &needle));
    assert!(!bits_equal_at(&haystack, 61, &needle));
}

#[test]
fn works_when_needle_reaches_haystack_end() {
    let haystack = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("001").unwrap();

    assert!(bits_equal_at(&haystack, 3, &needle));
}
