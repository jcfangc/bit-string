use crate::BitString;

#[test]
fn returns_len_for_empty_needle() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::new();

    assert_eq!(bits.rfind(needle.as_bit_str()), Some(bits.bit_len()));
}

#[test]
fn returns_none_when_needle_is_longer_than_self() {
    let bits = BitString::try_from("101").unwrap();
    let needle = BitString::try_from("1010").unwrap();

    assert_eq!(bits.rfind(needle.as_bit_str()), None);
}

#[test]
fn returns_last_match_index() {
    let bits = BitString::try_from("00110110").unwrap();
    let needle = BitString::try_from("110").unwrap();

    assert_eq!(bits.rfind(needle.as_bit_str()), Some(5));
}

#[test]
fn returns_start_when_only_match_is_at_start() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("101").unwrap();

    assert_eq!(bits.rfind(needle.as_bit_str()), Some(0));
}

#[test]
fn returns_end_match_when_match_is_at_end() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("001").unwrap();

    assert_eq!(bits.rfind(needle.as_bit_str()), Some(3));
}

#[test]
fn returns_none_when_needle_is_absent() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("111").unwrap();

    assert_eq!(bits.rfind(needle.as_bit_str()), None);
}

#[test]
fn returns_zero_when_needle_equals_self() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("101001").unwrap();

    assert_eq!(bits.rfind(needle.as_bit_str()), Some(0));
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(132);

    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);

    bits.set(127, true);
    bits.set(128, true);
    bits.set(129, true);

    let needle = BitString::try_from("01110").unwrap();

    assert_eq!(bits.rfind(needle.as_bit_str()), Some(126));
}
