use crate::BitString;
use alloc::string::ToString;

#[test]
fn pushes_true_to_empty_bit_string() {
    let mut bits = BitString::new();

    bits.push(true);

    assert_eq!(bits.bit_len(), 1);
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.to_string(), "1");
}

#[test]
fn pushes_false_to_empty_bit_string() {
    let mut bits = BitString::new();

    bits.push(false);

    assert_eq!(bits.bit_len(), 1);
    assert_eq!(bits.get(0), Some(false));
    assert_eq!(bits.to_string(), "0");
}

#[test]
fn appends_bits_without_changing_existing_prefix() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.push(false);
    bits.push(true);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "10101");
}

#[test]
fn pushes_across_word_boundary() {
    let mut bits = BitString::zeros(63);

    bits.push(true);
    bits.push(false);
    bits.push(true);

    assert_eq!(bits.bit_len(), 66);

    assert_eq!(bits.get(62), Some(false));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), Some(false));
    assert_eq!(bits.get(65), Some(true));
    assert_eq!(bits.get(66), None);
}

#[test]
fn pushing_false_preserves_count_ones() {
    let mut bits = BitString::try_from("1011").unwrap();

    bits.push(false);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.count_ones(), 3);
    assert_eq!(bits.count_zeros(), 2);
    assert_eq!(bits.to_string(), "10110");
}

#[test]
fn pushing_true_increments_count_ones() {
    let mut bits = BitString::try_from("1011").unwrap();

    bits.push(true);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.count_ones(), 4);
    assert_eq!(bits.count_zeros(), 1);
    assert_eq!(bits.to_string(), "10111");
}
