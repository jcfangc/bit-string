use alloc::string::ToString;

use crate::BitString;

#[test]
fn extends_with_owned_bool_items() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.extend([false, true, false]);

    assert_eq!(bits.len(), 6);
    assert_eq!(bits.to_string(), "101010");
}

#[test]
fn extends_with_borrowed_bool_items() {
    let mut bits = BitString::try_from("101").unwrap();
    let values = [false, true, false];

    bits.extend(values.iter());

    assert_eq!(bits.len(), 6);
    assert_eq!(bits.to_string(), "101010");
}

#[test]
fn extending_with_empty_iter_is_noop() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.extend(core::iter::empty::<bool>());

    assert_eq!(bits.len(), 3);
    assert_eq!(bits.to_string(), "101");
}

#[test]
fn extends_empty_bit_string() {
    let mut bits = BitString::new();

    bits.extend([true, false, true, true]);

    assert_eq!(bits.len(), 4);
    assert_eq!(bits.to_string(), "1011");
}

#[test]
fn extends_across_word_boundary() {
    let mut bits = BitString::zeros(63);

    bits.extend([true, false, true]);

    assert_eq!(bits.len(), 66);

    assert_eq!(bits.get(62), Some(false));
    assert_eq!(bits.get(63), Some(true));
    assert_eq!(bits.get(64), Some(false));
    assert_eq!(bits.get(65), Some(true));
    assert_eq!(bits.get(66), None);
}

#[test]
fn extend_updates_counts() {
    let mut bits = BitString::try_from("101").unwrap();

    bits.extend([true, false, true]);

    assert_eq!(bits.to_string(), "101101");
    assert_eq!(bits.count_ones(), 4);
    assert_eq!(bits.count_zeros(), 2);
}
