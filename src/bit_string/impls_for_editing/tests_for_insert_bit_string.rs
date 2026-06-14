use crate::BitString;
use alloc::string::ToString;

#[test]
fn inserts_bits_at_start() {
    let mut bits = BitString::try_from("101").unwrap();
    let rhs = BitString::try_from("00").unwrap();

    bits.insert_bit_string(0, &rhs);

    assert_eq!(bits.len(), 5);
    assert_eq!(bits.to_string(), "00101");
}

#[test]
fn inserts_bits_in_middle() {
    let mut bits = BitString::try_from("1001").unwrap();
    let rhs = BitString::try_from("11").unwrap();

    bits.insert_bit_string(2, &rhs);

    assert_eq!(bits.len(), 6);
    assert_eq!(bits.to_string(), "101101");
}

#[test]
fn inserts_bits_at_end() {
    let mut bits = BitString::try_from("101").unwrap();
    let rhs = BitString::try_from("01").unwrap();

    bits.insert_bit_string(bits.len(), &rhs);

    assert_eq!(bits.len(), 5);
    assert_eq!(bits.to_string(), "10101");
}

#[test]
fn inserts_bits_into_empty_bit_string() {
    let mut bits = BitString::new();
    let rhs = BitString::try_from("1011").unwrap();

    bits.insert_bit_string(0, &rhs);

    assert_eq!(bits.len(), 4);
    assert_eq!(bits.to_string(), "1011");
}

#[test]
fn inserting_empty_rhs_is_noop() {
    let mut bits = BitString::try_from("1011").unwrap();
    let rhs = BitString::new();

    bits.insert_bit_string(2, &rhs);

    assert_eq!(bits.len(), 4);
    assert_eq!(bits.to_string(), "1011");
}

#[test]
fn inserts_across_word_boundary() {
    let mut bits = BitString::zeros(130);
    bits.set(62, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(129, true);

    let rhs = BitString::try_from("10").unwrap();

    bits.insert_bit_string(63, &rhs);

    assert_eq!(bits.len(), 132);

    assert_eq!(bits.get(62), Some(true));
    assert_eq!(bits.get(63), Some(true)); // inserted 1
    assert_eq!(bits.get(64), Some(false)); // inserted 0
    assert_eq!(bits.get(65), Some(true)); // old 63
    assert_eq!(bits.get(66), Some(true)); // old 64
    assert_eq!(bits.get(131), Some(true)); // old 129
    assert_eq!(bits.get(132), None);
}

#[test]
fn insert_bit_string_updates_counts() {
    let mut bits = BitString::try_from("1001").unwrap();
    let rhs = BitString::try_from("111").unwrap();

    bits.insert_bit_string(2, &rhs);

    assert_eq!(bits.to_string(), "1011101");
    assert_eq!(bits.count_ones(), 5);
    assert_eq!(bits.count_zeros(), 2);
}

#[test]
#[should_panic(expected = "bit string insert index out of bounds")]
fn panics_when_inserting_past_len() {
    let mut bits = BitString::try_from("101").unwrap();
    let rhs = BitString::try_from("1").unwrap();

    bits.insert_bit_string(4, &rhs);
}
