use alloc::string::ToString;
use int_interval::UsizeCO;

use crate::BitString;

#[inline]
fn iv(start: usize, end_excl: usize) -> UsizeCO {
    UsizeCO::try_new(start, end_excl).unwrap()
}

#[test]
fn drains_prefix_interval() {
    let mut bits = BitString::try_from("101001").unwrap();

    let removed = bits.drain_interval(iv(0, 2));

    assert_eq!(removed.to_string(), "10");
    assert_eq!(bits.to_string(), "1001");
}

#[test]
fn drains_middle_interval() {
    let mut bits = BitString::try_from("101001").unwrap();

    let removed = bits.drain_interval(iv(2, 5));

    assert_eq!(removed.to_string(), "100");
    assert_eq!(bits.to_string(), "101");
}

#[test]
fn drains_suffix_interval() {
    let mut bits = BitString::try_from("101001").unwrap();

    let removed = bits.drain_interval(iv(3, 6));

    assert_eq!(removed.to_string(), "001");
    assert_eq!(bits.to_string(), "101");
}

#[test]
fn drains_entire_bit_string() {
    let mut bits = BitString::try_from("101001").unwrap();

    let removed = bits.drain_interval(iv(0, bits.bit_len()));

    assert_eq!(removed.to_string(), "101001");
    assert_eq!(bits.bit_len(), 0);
    assert_eq!(bits.to_string(), "");
    assert_eq!(bits.as_words().len(), 0);
}

#[test]
fn drains_across_word_boundary() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let removed = bits.drain_interval(iv(63, 66));

    assert_eq!(removed.bit_len(), 3);
    assert_eq!(removed.to_string(), "111");

    assert_eq!(bits.bit_len(), 127);
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(62), Some(false));
    assert_eq!(bits.get(63), Some(false));
    assert_eq!(bits.get(126), Some(true));
    assert_eq!(bits.get(127), None);
}

#[test]
fn drains_large_middle_interval_and_keeps_tail_order() {
    let mut bits = BitString::try_from("111000101011").unwrap();

    let removed = bits.drain_interval(iv(3, 9));

    assert_eq!(removed.to_string(), "000101");
    assert_eq!(bits.to_string(), "111011");
}

#[test]
fn removed_result_is_independent_from_original() {
    let mut bits = BitString::try_from("101001").unwrap();

    let mut removed = bits.drain_interval(iv(1, 4));

    assert_eq!(bits.to_string(), "101");
    assert_eq!(removed.to_string(), "010");

    bits.set(0, false);
    removed.set(1, false);

    assert_eq!(bits.to_string(), "001");
    assert_eq!(removed.to_string(), "000");
}

#[test]
#[should_panic(expected = "bit string interval out of bounds")]
fn panics_when_interval_end_exceeds_len() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.drain_interval(iv(1, 4));
}
