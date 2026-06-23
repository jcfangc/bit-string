use alloc::vec::Vec;
use int_interval::UsizeCO;

use crate::BitString;

#[test]
fn iterates_bits_in_order() {
    let bits = BitString::try_from("101001").unwrap();
    let v = bits.as_bitstr();

    let values: Vec<_> = v.iter().collect();

    assert_eq!(values, [true, false, true, false, false, true]);
}

#[test]
fn iterates_empty_view() {
    let bits = BitString::try_from("10110").unwrap();
    // Out-of-bounds slice clamps to empty.
    let v = bits.as_bitstr().slice(UsizeCO::try_new(10, 20).unwrap());
    let mut iter = v.iter();

    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.len(), 0);
}

#[test]
fn supports_double_ended_iteration() {
    let bits = BitString::try_from("101001").unwrap();
    let v = bits.as_bitstr();
    let mut iter = v.iter();

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
    let v = bits.as_bitstr();
    let mut iter = v.iter();

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
    let v = bits.as_bitstr();
    let mut iter = v.iter();

    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), Some(false));

    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn borrowed_into_iter_uses_bit_iterator() {
    let bits = BitString::try_from("1001").unwrap();
    let v = bits.as_bitstr();

    let values: Vec<_> = (&v).into_iter().collect();

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

    let v = bits.as_bitstr();
    let values: Vec<_> = v.iter().enumerate().map(|(i, b)| (i, b)).collect();

    assert_eq!(values.len(), 130);
    assert!(values[0].1);
    assert!(values[63].1);
    assert!(values[64].1);
    assert!(values[65].1);
    assert!(values[129].1);

    assert!(!values[1].1);
    assert!(!values[62].1);
    assert!(!values[66].1);
    assert!(!values[128].1);
}

#[test]
fn iterates_offset_view() {
    let bits = BitString::try_from("101100").unwrap();
    // bits: 1 0 1 1 0 0
    // view bits 1..5 → 0 1 1 0
    let v = bits.as_bitstr().slice(UsizeCO::try_new(1, 5).unwrap());

    let values: Vec<_> = v.iter().collect();
    assert_eq!(values, [false, true, true, false]);
}

#[test]
fn double_ended_on_offset_view() {
    let bits = BitString::try_from("101100").unwrap();
    // view bits 2..6 → 1 1 0 0
    let v = bits.as_bitstr().slice(UsizeCO::try_new(2, 6).unwrap());
    let mut iter = v.iter();

    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), None);
}

#[test]
fn into_iter_on_offset_view() {
    let bits = BitString::try_from("101100").unwrap();
    let v = bits.as_bitstr().slice(UsizeCO::try_new(1, 4).unwrap());

    let values: Vec<_> = (&v).into_iter().collect();
    assert_eq!(values, [false, true, true]);
}

#[test]
fn to_bool_vec_on_full_view() {
    let bits = BitString::try_from("11010").unwrap();
    let v = bits.as_bitstr();

    assert_eq!(v.to_bool_vec(), [true, true, false, true, false]);
}

#[test]
fn to_bool_vec_on_offset_view() {
    let bits = BitString::try_from("11010").unwrap();
    // view bits 1..4 → 1 0 1
    let v = bits.as_bitstr().slice(UsizeCO::try_new(1, 4).unwrap());

    assert_eq!(v.to_bool_vec(), [true, false, true]);
}

#[test]
fn for_loop_over_view() {
    let bits = BitString::try_from("101").unwrap();
    let v = bits.as_bitstr();

    let mut acc = Vec::new();
    for b in &v {
        acc.push(b);
    }
    assert_eq!(acc, [true, false, true]);
}

#[test]
fn exact_size_is_correct() {
    let bits = BitString::try_from("1010011100").unwrap();
    let v = bits.as_bitstr();

    let mut iter = v.iter();
    assert_eq!(iter.len(), 10);

    for _ in 0..10 {
        let prev = iter.len();
        assert!(iter.next().is_some());
        assert_eq!(iter.len(), prev - 1);
    }
    assert_eq!(iter.len(), 0);
}
