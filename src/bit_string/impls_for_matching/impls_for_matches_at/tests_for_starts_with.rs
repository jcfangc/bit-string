use crate::BitString;

#[test]
fn returns_true_for_empty_prefix() {
    let bits = BitString::try_from("101001").unwrap();
    let prefix = BitString::new();

    assert!(bits.starts_with(&prefix));
}

#[test]
fn returns_true_for_matching_prefix() {
    let bits = BitString::try_from("101001").unwrap();
    let prefix = BitString::try_from("101").unwrap();

    assert!(bits.starts_with(&prefix));
}

#[test]
fn returns_true_for_full_self_prefix() {
    let bits = BitString::try_from("101001").unwrap();
    let prefix = BitString::try_from("101001").unwrap();

    assert!(bits.starts_with(&prefix));
}

#[test]
fn returns_false_for_non_matching_prefix() {
    let bits = BitString::try_from("101001").unwrap();
    let prefix = BitString::try_from("100").unwrap();

    assert!(!bits.starts_with(&prefix));
}

#[test]
fn returns_false_when_prefix_is_longer_than_self() {
    let bits = BitString::try_from("101").unwrap();
    let prefix = BitString::try_from("1010").unwrap();

    assert!(!bits.starts_with(&prefix));
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);

    let mut prefix = BitString::zeros(66);
    prefix.set(0, true);
    prefix.set(63, true);
    prefix.set(64, true);
    prefix.set(65, true);

    assert!(bits.starts_with(&prefix));

    prefix.set(62, true);

    assert!(!bits.starts_with(&prefix));
}
