use super::*;
use int_interval::UsizeCO;

#[test]
fn attack_slice_interval_clamping() {
    let bits = bs("11001"); // len 5

    // Interval completely beyond
    let s = bits.slice(UsizeCO::try_new(10, 13).unwrap());
    assert!(s.is_empty());

    // Interval partially beyond
    let s = bits.slice(UsizeCO::checked_from_start_len(3, 10).unwrap());
    assert_eq!(s.to_string(), "01");

    // Non-empty slice from middle
    let s = bits.slice(UsizeCO::checked_from_start_len(2, 1).unwrap());
    assert_eq!(s.bit_len(), 1);

    // Full interval
    let s = bits.slice(UsizeCO::checked_from_start_len(0, 5).unwrap());
    assert_eq!(s.to_string(), "11001");
}

#[test]
fn attack_slice_from_until_clamping() {
    let bits = bs("10101");
    assert_eq!(bits.slice_from(0).to_string(), "10101");
    assert_eq!(bits.slice_from(5).to_string(), "");
    assert_eq!(bits.slice_from(usize::MAX).to_string(), "");
    assert_eq!(bits.slice_until(0).to_string(), "");
    assert_eq!(bits.slice_until(5).to_string(), "10101");
    assert_eq!(bits.slice_until(usize::MAX).to_string(), "10101");
}

#[test]
fn attack_drain_interval_edge() {
    let bits = bs("110011");

    // Drain interval beyond bounds
    let d = bits.drain_interval(UsizeCO::checked_from_start_len(10, 5).unwrap());
    assert_eq!(d.to_string(), "110011");

    // Drain everything
    let d = bits.drain_interval(UsizeCO::checked_from_start_len(0, 6).unwrap());
    assert!(d.is_empty());

    // Drain from middle
    let d = bits.drain_interval(UsizeCO::checked_from_start_len(2, 2).unwrap());
    assert_eq!(d.to_string(), "1111");
}

#[test]
fn attack_drain_interval_assign_edge() {
    // Drain from middle mutable
    let mut bits = bs("110011");
    bits.drain_interval_assign(UsizeCO::checked_from_start_len(2, 2).unwrap());
    assert_eq!(bits.to_string(), "1111");
    assert!(view_has_same_invariants(&bits));

    // Drain everything mutable
    let mut bits = bs("101");
    bits.drain_interval_assign(UsizeCO::checked_from_start_len(0, 3).unwrap());
    assert!(bits.is_empty());
    assert!(view_has_same_invariants(&bits));
}
