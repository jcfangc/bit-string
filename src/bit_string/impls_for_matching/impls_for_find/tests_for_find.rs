use crate::BitString;

#[test]
fn empty_needle_matches_at_zero() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::new();

    assert_eq!(bits.find(&needle), Some(0));
}

#[test]
fn returns_none_when_needle_is_longer() {
    let bits = BitString::try_from("101").unwrap();
    let needle = BitString::try_from("0101").unwrap();

    assert_eq!(bits.find(&needle), None);
}

#[test]
fn finds_needle_at_start() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("101").unwrap();

    assert_eq!(bits.find(&needle), Some(0));
}

#[test]
fn finds_needle_in_middle() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("100").unwrap();

    assert_eq!(bits.find(&needle), Some(2));
}

#[test]
fn finds_needle_at_end() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("001").unwrap();

    assert_eq!(bits.find(&needle), Some(3));
}

#[test]
fn returns_none_when_needle_is_absent() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("111").unwrap();

    assert_eq!(bits.find(&needle), None);
}

#[test]
fn returns_first_match_when_needle_occurs_multiple_times() {
    let bits = BitString::try_from("101101101").unwrap();
    let needle = BitString::try_from("101").unwrap();

    assert_eq!(bits.find(&needle), Some(0));
}

#[test]
fn finds_match_across_word_boundary() {
    let mut bits = BitString::zeros(130);
    let needle = BitString::try_from("111").unwrap();

    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);

    assert_eq!(bits.find(&needle), Some(63));
}
