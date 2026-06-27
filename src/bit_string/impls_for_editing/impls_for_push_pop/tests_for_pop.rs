use crate::BitString;
use alloc::string::ToString;

#[test]
fn returns_none_for_empty_bit_string() {
    let mut bits = BitString::new();

    assert_eq!(bits.pop(), None);
    assert_eq!(bits.bit_len(), 0);
    assert_eq!(bits.to_string(), "");
}

#[test]
fn pops_bits_from_back_in_order() {
    let mut bits = BitString::try_from("101001").unwrap();

    assert_eq!(bits.pop(), Some(true));
    assert_eq!(bits.to_string(), "10100");

    assert_eq!(bits.pop(), Some(false));
    assert_eq!(bits.to_string(), "1010");

    assert_eq!(bits.pop(), Some(false));
    assert_eq!(bits.to_string(), "101");

    assert_eq!(bits.pop(), Some(true));
    assert_eq!(bits.to_string(), "10");

    assert_eq!(bits.pop(), Some(false));
    assert_eq!(bits.to_string(), "1");

    assert_eq!(bits.pop(), Some(true));
    assert_eq!(bits.to_string(), "");

    assert_eq!(bits.pop(), None);
}

#[test]
fn shrinks_when_crossing_word_boundary() {
    let mut bits = BitString::ones(65);

    assert_eq!(bits.words().len(), 2);
    assert_eq!(bits.pop(), Some(true));

    assert_eq!(bits.bit_len(), 64);
    assert_eq!(bits.words().len(), 1);
    assert_eq!(bits.count_ones(), 64);
    assert_eq!(bits.to_string(), "1".repeat(64));
}

#[test]
fn preserves_lower_bits_after_pop() {
    let mut bits =
        BitString::try_from("10000000000000000000000000000000000000000000000000000000000000001")
            .unwrap();

    assert_eq!(bits.bit_len(), 65);
    assert_eq!(bits.pop(), Some(true));

    assert_eq!(bits.bit_len(), 64);
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(63), Some(false));
    assert_eq!(bits.get(64), None);
}

#[test]
fn removes_last_zero_bit() {
    let mut bits = BitString::try_from("1010").unwrap();

    assert_eq!(bits.pop(), Some(false));
    assert_eq!(bits.bit_len(), 3);
    assert_eq!(bits.to_string(), "101");
}

#[test]
fn repeated_pop_eventually_clears_storage() {
    let mut bits = BitString::ones(130);

    for _ in 0..130 {
        assert_eq!(bits.pop(), Some(true));
    }

    assert_eq!(bits.pop(), None);
    assert_eq!(bits.bit_len(), 0);
    assert_eq!(bits.words().len(), 0);
    assert_eq!(bits.to_string(), "");
}
