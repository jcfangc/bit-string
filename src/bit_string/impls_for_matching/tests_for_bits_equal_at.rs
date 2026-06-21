use crate::BitString;

#[test]
fn returns_true_when_needle_matches_at_offset() {
    let haystack = BitString::try_from("00110110").unwrap();
    let needle = BitString::try_from("110").unwrap();

    assert!(haystack.bits_equal_at(2, &needle));
    assert!(haystack.bits_equal_at(5, &needle));
}

#[test]
fn returns_false_when_needle_differs_at_offset() {
    let haystack = BitString::try_from("00110110").unwrap();
    let needle = BitString::try_from("110").unwrap();

    assert!(!haystack.bits_equal_at(0, &needle));
    assert!(!haystack.bits_equal_at(1, &needle));
    assert!(!haystack.bits_equal_at(3, &needle));
}

#[test]
fn empty_needle_matches_at_valid_boundary_offsets() {
    let haystack = BitString::try_from("101001").unwrap();
    let needle = BitString::new();

    assert!(haystack.bits_equal_at(0, &needle));
    assert!(haystack.bits_equal_at(3, &needle));
    assert!(haystack.bits_equal_at(haystack.bit_len(), &needle));
}

#[test]
fn works_across_word_boundaries() {
    let mut haystack = BitString::zeros(130);

    haystack.set(63, true);
    haystack.set(64, true);
    haystack.set(65, true);

    let needle = BitString::try_from("01110").unwrap();

    assert!(haystack.bits_equal_at(62, &needle));
    assert!(!haystack.bits_equal_at(61, &needle));
}

#[test]
fn works_when_needle_reaches_haystack_end() {
    let haystack = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("001").unwrap();

    assert!(haystack.bits_equal_at(3, &needle));
}
