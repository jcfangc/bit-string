use crate::BitString;

#[test]
fn returns_none_for_empty_bit_string() {
    let bits = BitString::new();

    assert_eq!(bits.get(0), None);
}

#[test]
fn returns_bits_by_index() {
    let bits = BitString::try_from("101001").unwrap();

    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(1), Some(false));
    assert_eq!(bits.get(2), Some(true));
    assert_eq!(bits.get(3), Some(false));
    assert_eq!(bits.get(4), Some(false));
    assert_eq!(bits.get(5), Some(true));
}

#[test]
fn returns_none_at_len_and_beyond() {
    let bits = BitString::try_from("101001").unwrap();

    assert_eq!(bits.get(bits.bit_len()), None);
    assert_eq!(bits.get(bits.bit_len() + 1), None);
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(1), Some(false));
    assert_eq!(bits.get(62), Some(false));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), Some(true));
    assert_eq!(bits.get(66), Some(false));
    assert_eq!(bits.get(128), Some(false));
    assert_eq!(bits.get(129), Some(true));
    assert_eq!(bits.get(130), None);
}
