use alloc::string::ToString;
use int_interval::UsizeCO;

use crate::BitString;

#[inline]
fn iv(start: usize, end_excl: usize) -> UsizeCO {
    UsizeCO::try_new(start, end_excl).unwrap()
}

#[test]
fn slices_prefix_interval() {
    let bits = BitString::try_from("101001").unwrap();

    let sliced = bits.slice(iv(0, 2));

    assert_eq!(sliced.to_string(), "10");
    assert_eq!(bits.to_string(), "101001");
}

#[test]
fn slices_middle_interval() {
    let bits = BitString::try_from("101001").unwrap();

    let sliced = bits.slice(iv(2, 5));

    assert_eq!(sliced.to_string(), "100");
    assert_eq!(bits.to_string(), "101001");
}

#[test]
fn slices_suffix_interval() {
    let bits = BitString::try_from("101001").unwrap();

    let sliced = bits.slice(iv(3, 6));

    assert_eq!(sliced.to_string(), "001");
    assert_eq!(bits.to_string(), "101001");
}

#[test]
fn slices_entire_bit_string() {
    let bits = BitString::try_from("101001").unwrap();

    let sliced = bits.slice(iv(0, bits.bit_len()));

    assert_eq!(sliced.bit_len(), bits.bit_len());
    assert_eq!(sliced.to_string(), "101001");
    assert_eq!(bits.to_string(), "101001");
}

#[test]
fn slices_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let sliced = bits.slice(iv(63, 66));

    assert_eq!(sliced.bit_len(), 3);
    assert_eq!(sliced.to_string(), "111");
}

#[test]
fn slices_large_interval_and_preserves_order() {
    let bits = BitString::try_from("111000101011").unwrap();

    let sliced = bits.slice(iv(3, 9));

    assert_eq!(sliced.to_string(), "000101");
}

#[test]
fn sliced_result_is_independent_from_original() {
    let mut bits = BitString::try_from("101001").unwrap();

    let mut sliced = bits.slice(iv(1, 4));

    assert_eq!(bits.to_string(), "101001");
    assert_eq!(sliced.to_string(), "010");

    bits.set(1, true);
    sliced.set(1, false);

    assert_eq!(bits.to_string(), "111001");
    assert_eq!(sliced.to_string(), "000");
}

#[test]
fn slices_single_bit_interval() {
    let bits = BitString::try_from("101001").unwrap();

    let sliced = bits.slice(iv(2, 3));

    assert_eq!(sliced.bit_len(), 1);
    assert_eq!(sliced.to_string(), "1");
}

#[test]
fn clamps_interval_when_end_exceeds_len() {
    let bits = BitString::try_from("101").unwrap();

    let sliced = bits.slice(iv(1, 4));

    assert_eq!(sliced.bit_len(), 2);
    assert_eq!(sliced.to_string(), "01");
}
