use alloc::vec::Vec;

use crate::BitString;

#[test]
fn iterates_bits_in_order() {
    let bits = BitString::try_from("101001").unwrap();

    let values: Vec<_> = bits.iter().collect();

    assert_eq!(values, [true, false, true, false, false, true]);
}

#[test]
fn iterates_empty_bit_string() {
    let bits = BitString::new();
    let mut iter = bits.iter();

    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.len(), 0);
}

#[test]
fn supports_double_ended_iteration() {
    let bits = BitString::try_from("101001").unwrap();
    let mut iter = bits.iter();

    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(true));
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn size_hint_tracks_remaining_bits() {
    let bits = BitString::try_from("101001").unwrap();
    let mut iter = bits.iter();

    assert_eq!(iter.size_hint(), (6, Some(6)));
    assert_eq!(iter.len(), 6);

    assert_eq!(iter.next(), Some(true));

    assert_eq!(iter.size_hint(), (5, Some(5)));
    assert_eq!(iter.len(), 5);

    assert_eq!(iter.next_back(), Some(true));

    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.len(), 4);
}

#[test]
fn remains_fused_after_exhaustion() {
    let bits = BitString::try_from("10").unwrap();
    let mut iter = bits.iter();

    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), Some(false));

    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn borrowed_into_iter_uses_bit_iterator() {
    let bits = BitString::try_from("1001").unwrap();

    let values: Vec<_> = (&bits).into_iter().collect();

    assert_eq!(values, [true, false, false, true]);
}

#[test]
fn works_across_word_boundaries() {
    let mut bits = BitString::zeros(130);

    bits.set(0, true);
    bits.set(63, true);
    bits.set(64, true);
    bits.set(65, true);
    bits.set(129, true);

    let values: Vec<_> = bits.iter().collect();

    assert_eq!(values.len(), 130);
    assert!(values[0]);
    assert!(values[63]);
    assert!(values[64]);
    assert!(values[65]);
    assert!(values[129]);

    assert!(!values[1]);
    assert!(!values[62]);
    assert!(!values[66]);
    assert!(!values[128]);
}
