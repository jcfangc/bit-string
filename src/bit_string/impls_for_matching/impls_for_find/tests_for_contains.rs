use crate::BitString;

#[test]
fn returns_true_for_empty_needle() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::new();

    assert!(bits.contains(&needle));
}

#[test]
fn returns_true_when_needle_is_found_at_start() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("101").unwrap();

    assert!(bits.contains(&needle));
}

#[test]
fn returns_true_when_needle_is_found_in_middle() {
    let bits = BitString::try_from("00110110").unwrap();
    let needle = BitString::try_from("110").unwrap();

    assert!(bits.contains(&needle));
}

#[test]
fn returns_true_when_needle_is_found_at_end() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("001").unwrap();

    assert!(bits.contains(&needle));
}

#[test]
fn returns_true_when_needle_equals_self() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("101001").unwrap();

    assert!(bits.contains(&needle));
}

#[test]
fn returns_false_when_needle_is_absent() {
    let bits = BitString::try_from("101001").unwrap();
    let needle = BitString::try_from("111").unwrap();

    assert!(!bits.contains(&needle));
}

#[test]
fn returns_false_when_needle_is_longer_than_self() {
    let bits = BitString::try_from("101").unwrap();
    let needle = BitString::try_from("1010").unwrap();

    assert!(!bits.contains(&needle));
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(130);

    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);

    let present = BitString::try_from("01110").unwrap();
    let absent = BitString::try_from("11110").unwrap();

    assert!(bits.contains(&present));
    assert!(!bits.contains(&absent));
}
