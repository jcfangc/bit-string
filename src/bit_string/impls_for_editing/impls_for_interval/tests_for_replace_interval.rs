use crate::BitString;
use alloc::string::ToString;

use int_interval::UsizeCO;

#[inline]
fn iv(start: usize, end_excl: usize) -> UsizeCO {
    UsizeCO::try_new(start, end_excl).unwrap()
}

#[test]
fn replaces_interval_with_same_length_bits() {
    let mut bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("1010").unwrap();

    bits.replace_interval(iv(2, 6), &replacement);

    assert_eq!(bits.bit_len(), 8);
    assert_eq!(bits.to_string(), "00101000");
}

#[test]
fn replaces_interval_with_shorter_bits() {
    let mut bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::try_from("10").unwrap();

    bits.replace_interval(iv(2, 6), &replacement);

    assert_eq!(bits.bit_len(), 6);
    assert_eq!(bits.to_string(), "001000");
}

#[test]
fn replaces_interval_with_longer_bits() {
    let mut bits = BitString::try_from("001100").unwrap();
    let replacement = BitString::try_from("10101").unwrap();

    bits.replace_interval(iv(2, 4), &replacement);

    assert_eq!(bits.bit_len(), 9);
    assert_eq!(bits.to_string(), "001010100");
}

#[test]
fn replaces_interval_with_empty_bits() {
    let mut bits = BitString::try_from("00111100").unwrap();
    let replacement = BitString::new();

    bits.replace_interval(iv(2, 6), &replacement);

    assert_eq!(bits.bit_len(), 4);
    assert_eq!(bits.to_string(), "0000");
}

#[test]
fn replaces_interval_at_start() {
    let mut bits = BitString::try_from("111000").unwrap();
    let replacement = BitString::try_from("00").unwrap();

    bits.replace_interval(iv(0, 3), &replacement);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "00000");
}

#[test]
fn replaces_interval_at_end() {
    let mut bits = BitString::try_from("000111").unwrap();
    let replacement = BitString::try_from("10").unwrap();

    bits.replace_interval(iv(3, 6), &replacement);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "00010");
}

#[test]
fn replaces_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    bits.set(62, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let replacement = BitString::try_from("01").unwrap();

    bits.replace_interval(iv(63, 65), &replacement);

    assert_eq!(bits.bit_len(), 130);

    assert_eq!(bits.get(62), Some(true));
    assert_eq!(bits.get(63), Some(false)); // replacement 0
    assert_eq!(bits.get(64), Some(true)); // replacement 1
    assert_eq!(bits.get(65), Some(true)); // old 65
    assert_eq!(bits.get(129), Some(true));
    assert_eq!(bits.get(130), None);
}

#[test]
fn replace_interval_updates_counts() {
    let mut bits = BitString::try_from("111000").unwrap();
    let replacement = BitString::try_from("101").unwrap();

    bits.replace_interval(iv(1, 5), &replacement);

    assert_eq!(bits.to_string(), "11010");
    assert_eq!(bits.count_ones(), 3);
    assert_eq!(bits.count_zeros(), 2);
}

#[test]
#[should_panic(expected = "bit string interval out of bounds")]
fn panics_when_interval_end_is_out_of_bounds() {
    let mut bits = BitString::try_from("101").unwrap();
    let replacement = BitString::try_from("0").unwrap();

    bits.replace_interval(iv(1, 4), &replacement);
}
