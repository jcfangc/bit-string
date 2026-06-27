use super::*;

#[test]
fn attack_retain_all_none() {
    let bits = bs("1010101");

    // Retain all
    let mut copy = bits.clone();
    copy.retain(|_| true);
    assert_eq!(copy, bits);
    assert!(view_has_same_invariants(&copy));

    // Retain none
    copy.retain(|_| false);
    assert!(copy.is_empty());
    assert!(view_has_same_invariants(&copy));

    // Retain only ones
    let mut bits = bs("1010101");
    bits.retain(|b| b);
    assert_eq!(bits.to_string(), "1111");
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_retain_alternating() {
    let mut bits = BitString::ones(200);
    bits.retain(|b| b); // identity
    assert_eq!(bits.bit_len(), 200);
    assert_eq!(bits.count_ones(), 200);
    assert!(view_has_same_invariants(&bits));
}
