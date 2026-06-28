use crate::BitString;

#[test]
fn returns_true_when_needle_matches_at_offset() {
    let haystack = BitString::try_from("00110110").unwrap();
    let haystack = haystack.as_bit_str();
    let needle = BitString::try_from("110").unwrap();

    assert!(haystack.bits_equal_at_inner::<false, false>(2, needle.as_bit_str()));
    assert!(haystack.bits_equal_at_inner::<false, false>(5, needle.as_bit_str()));
}

#[test]
fn returns_false_when_needle_differs_at_offset() {
    let haystack = BitString::try_from("00110110").unwrap();
    let haystack = haystack.as_bit_str();
    let needle = BitString::try_from("110").unwrap();

    assert!(!haystack.bits_equal_at_inner::<false, false>(0, needle.as_bit_str()));
    assert!(!haystack.bits_equal_at_inner::<false, false>(1, needle.as_bit_str()));
    assert!(!haystack.bits_equal_at_inner::<false, false>(3, needle.as_bit_str()));
}

#[test]
fn empty_needle_matches_at_valid_boundary_offsets() {
    let haystack = BitString::try_from("101001").unwrap();
    let haystack = haystack.as_bit_str();
    let needle = BitString::new();

    assert!(haystack.bits_equal_at_inner::<false, false>(0, needle.as_bit_str()));
    assert!(haystack.bits_equal_at_inner::<false, false>(3, needle.as_bit_str()));
    assert!(haystack.bits_equal_at_inner::<false, false>(haystack.bit_len(), needle.as_bit_str()));
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(130);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);

    let haystack = bits.as_bit_str();
    let needle = BitString::try_from("01110").unwrap();

    assert!(haystack.bits_equal_at_inner::<false, false>(62, needle.as_bit_str()));
    assert!(!haystack.bits_equal_at_inner::<false, false>(61, needle.as_bit_str()));
}

#[test]
fn works_when_needle_reaches_haystack_end() {
    let haystack = BitString::try_from("101001").unwrap();
    let haystack = haystack.as_bit_str();
    let needle = BitString::try_from("001").unwrap();

    assert!(haystack.bits_equal_at_inner::<false, false>(3, needle.as_bit_str()));
}
