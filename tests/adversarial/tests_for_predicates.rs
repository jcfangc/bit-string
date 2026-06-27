use super::*;
use int_interval::UsizeCO;

#[test]
fn attack_predicates_corner() {
    assert!(!BitString::new().any());
    assert!(BitString::new().all()); // vacuously true
    assert!(BitString::new().is_all_zeros());
    assert!(BitString::new().is_all_ones()); // also vacuously true

    let z = BitString::zeros(10);
    assert!(!z.any());
    assert!(!z.all());
    assert!(z.is_all_zeros());
    assert!(!z.is_all_ones());

    let o = BitString::ones(10);
    assert!(o.any());
    assert!(o.all());
    assert!(!o.is_all_zeros());
    assert!(o.is_all_ones());
}

// ===========================================================================
// K. BitStr predicates on unaligned views
// ===========================================================================

#[test]
fn attack_predicates_unaligned_all_ones() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 10).unwrap());
    assert!(view.all());
    assert!(view.is_all_ones());
    assert!(!view.is_all_zeros());
    assert!(view.any());
}

#[test]
fn attack_predicates_unaligned_all_zeros() {
    let a = bs(&cat(&[
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 10).unwrap());
    assert!(!view.any());
    assert!(!view.all());
    assert!(view.is_all_zeros());
}

#[test]
fn attack_predicates_unaligned_empty_view() {
    let bits = bs("10101");
    // empty view via sliced at boundary
    let v1 = bits.as_bit_str().slice_from(5);
    assert!(v1.is_empty());
    assert!(v1.all());
    assert!(v1.is_all_zeros());
    assert!(v1.is_all_ones());
    assert!(!v1.any());
}
