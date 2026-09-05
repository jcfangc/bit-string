use alloc::string::ToString;
use alloc::vec::Vec;

use crate::{BitString, WORD_BITS};

#[test]
fn builds_empty_bit_string_from_empty_iter() {
    let bits = BitString::from_bool_iter(core::iter::empty());

    assert_eq!(bits.bit_len(), 0);
    assert!(bits.is_empty());
    assert_eq!(bits.to_string(), "");
    assert_eq!(bits.count_ones(), 0);
    assert_eq!(bits.count_zeros(), 0);
}

#[test]
fn builds_single_partial_word() {
    let bits = BitString::from_bool_iter([true, false, true, true, false]);

    assert_eq!(bits.bit_len(), 5);
    assert_eq!(bits.to_string(), "10110");
    assert_eq!(bits.count_ones(), 3);
    assert_eq!(bits.count_zeros(), 2);

    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(1), Some(false));
    assert_eq!(bits.get(2), Some(true));
    assert_eq!(bits.get(3), Some(true));
    assert_eq!(bits.get(4), Some(false));
    assert_eq!(bits.get(5), None);
}

#[test]
fn builds_exactly_one_full_word() {
    let values = (0..WORD_BITS).map(|index| index % 2 == 0);
    let bits = BitString::from_bool_iter(values);

    assert_eq!(bits.bit_len(), WORD_BITS);
    assert_eq!(bits.count_ones(), WORD_BITS / 2);
    assert_eq!(bits.count_zeros(), WORD_BITS / 2);

    for index in 0..WORD_BITS {
        assert_eq!(bits.get(index), Some(index % 2 == 0), "index={index}");
    }
}

#[test]
fn builds_across_word_boundary() {
    let len = WORD_BITS + 3;
    let values = (0..len).map(|index| matches!(index, 0 | 63 | 64 | 66));
    let bits = BitString::from_bool_iter(values);

    assert_eq!(bits.bit_len(), len);
    assert_eq!(bits.count_ones(), 4);

    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), Some(false));
    assert_eq!(bits.get(66), Some(true));
    assert_eq!(bits.get(67), None);
}

#[test]
fn preserves_iteration_order() {
    let values = [
        false, true, false, false, true, true, false, true, true, false,
    ];

    let bits = BitString::from_bool_iter(values);

    assert_eq!(bits.iter().collect::<Vec<_>>(), values);
    assert_eq!(bits.to_string(), "0100110110");
}

#[test]
fn leaves_unused_tail_bits_zero() {
    let bits = BitString::from_bool_iter([true]);

    assert_eq!(bits.bit_len(), 1);
    assert_eq!(bits.words(), &[1]);
}
