use crate::BitString;
use alloc::string::ToString;

#[test]
fn appending_empty_bit_string_is_noop() {
    let mut bits = BitString::try_from("1010").unwrap();
    let rhs = BitString::new();

    bits.push_bits(&rhs);

    assert_eq!(bits.len(), 4);
    assert_eq!(bits.to_string(), "1010");
}

#[test]
fn appends_to_empty_bit_string() {
    let mut bits = BitString::new();
    let rhs = BitString::try_from("101001").unwrap();

    bits.push_bits(&rhs);

    assert_eq!(bits.len(), 6);
    assert_eq!(bits.to_string(), "101001");
}

#[test]
fn appends_non_empty_bit_strings() {
    let mut bits = BitString::try_from("1010").unwrap();
    let rhs = BitString::try_from("011").unwrap();

    bits.push_bits(&rhs);

    assert_eq!(bits.len(), 7);
    assert_eq!(bits.to_string(), "1010011");
}

#[test]
fn appends_when_lhs_ends_before_word_boundary() {
    let mut bits = BitString::zeros(63);
    let rhs = BitString::try_from("11").unwrap();

    bits.set(0, true);
    bits.set(62, true);

    bits.push_bits(&rhs);

    assert_eq!(bits.len(), 65);
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(61), Some(false));
    assert_eq!(bits.get(62), Some(true));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), None);
}

#[test]
fn appends_when_lhs_is_word_aligned() {
    let mut bits = BitString::ones(64);
    let rhs = BitString::try_from("01").unwrap();

    bits.push_bits(&rhs);

    assert_eq!(bits.len(), 66);
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), Some(false));
    assert_eq!(bits.get(65), Some(true));
    assert_eq!(bits.get(66), None);
}

#[test]
fn appends_rhs_spanning_multiple_words() {
    let mut bits = BitString::try_from("10").unwrap();
    let mut rhs = BitString::zeros(130);

    rhs.set(0, true);
    rhs.set(64, true);
    rhs.set(129, true);

    bits.push_bits(&rhs);

    assert_eq!(bits.len(), 132);
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(1), Some(false));

    assert_eq!(bits.get(2), Some(true));
    assert_eq!(bits.get(66), Some(true));
    assert_eq!(bits.get(131), Some(true));

    assert_eq!(bits.get(65), Some(false));
    assert_eq!(bits.get(130), Some(false));
    assert_eq!(bits.get(132), None);
}

#[test]
fn does_not_mutate_rhs() {
    let mut bits = BitString::try_from("10").unwrap();
    let rhs = BitString::try_from("0110").unwrap();

    bits.push_bits(&rhs);

    assert_eq!(bits.to_string(), "100110");
    assert_eq!(rhs.to_string(), "0110");
}
