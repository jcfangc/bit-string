use alloc::string::ToString;

use crate::BitString;

#[test]
fn strips_empty_suffix_by_returning_self_copy() {
    let bits = BitString::try_from("101001").unwrap();
    let suffix = BitString::new();

    let stripped = bits.strip_suffix(&suffix).unwrap();

    assert_eq!(stripped.to_string(), "101001");
}

#[test]
fn strips_matching_suffix() {
    let bits = BitString::try_from("101001").unwrap();
    let suffix = BitString::try_from("001").unwrap();

    let stripped = bits.strip_suffix(&suffix).unwrap();

    assert_eq!(stripped.to_string(), "101");
}

#[test]
fn strips_full_self_suffix_to_empty() {
    let bits = BitString::try_from("101001").unwrap();
    let suffix = BitString::try_from("101001").unwrap();

    let stripped = bits.strip_suffix(&suffix).unwrap();

    assert!(stripped.is_empty());
    assert_eq!(stripped.to_string(), "");
}

#[test]
fn returns_none_for_non_matching_suffix() {
    let bits = BitString::try_from("101001").unwrap();
    let suffix = BitString::try_from("101").unwrap();

    assert_eq!(bits.strip_suffix(&suffix), None);
}

#[test]
fn returns_none_when_suffix_is_longer_than_self() {
    let bits = BitString::try_from("101").unwrap();
    let suffix = BitString::try_from("0101").unwrap();

    assert_eq!(bits.strip_suffix(&suffix), None);
}

#[test]
fn result_is_independent_from_original() {
    let mut bits = BitString::try_from("101001").unwrap();
    let suffix = BitString::try_from("001").unwrap();

    let mut stripped = bits.strip_suffix(&suffix).unwrap();

    bits.set(0, false);
    stripped.set(0, false);

    assert_eq!(bits.to_string(), "001001");
    assert_eq!(stripped.to_string(), "001");
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let mut suffix = BitString::zeros(67);
    suffix.set(0, true);
    suffix.set(1, true);
    suffix.set(2, true);
    suffix.set(66, true);

    let stripped = bits.strip_suffix(&suffix).unwrap();

    assert_eq!(stripped.bit_len(), 63);
    assert_eq!(stripped.get(0), Some(true));
    assert_eq!(stripped.get(62), Some(false));
}

#[test]
fn returns_none_for_almost_matching_cross_word_suffix() {
    let mut bits = BitString::zeros(130);

    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let mut suffix = BitString::zeros(67);
    suffix.set(0, true);
    suffix.set(1, true);
    suffix.set(2, true);
    suffix.set(65, true);

    assert_eq!(bits.strip_suffix(&suffix), None);
}
