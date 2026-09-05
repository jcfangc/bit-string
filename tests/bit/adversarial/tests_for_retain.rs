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

// ===========================================================================
// H. retain at word boundaries
// ===========================================================================

#[test]
fn attack_retain_crossing_word_boundary() {
    let mut bits = BitString::zeros(200);
    for i in (63..200).step_by(3) {
        bits.set(i, true);
    }
    let ones_count_before = bits.count_ones();
    bits.retain(|b| b);
    assert_eq!(bits.count_ones(), ones_count_before);
    assert!(view_has_same_invariants(&bits));
    assert!(bits.is_all_ones());
}

#[test]
fn attack_retain_all_false_clears_long() {
    let mut bits = BitString::ones(130);
    bits.retain(|_| false);
    assert!(bits.is_empty());
    assert!(view_has_same_invariants(&bits));
}
