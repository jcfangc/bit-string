use super::*;
use core::cmp::Ordering;

#[test]
fn attack_ord_different_lengths() {
    // Shorter is less than longer when common prefix equal
    let a = bs("101");
    let b = bs("1010");
    assert!(a < b);

    // First differing bit determines order
    let a = bs("100");
    let b = bs("101");
    assert!(a < b);

    let a = bs("110");
    let b = bs("101");
    assert!(a > b);
}

#[test]
fn attack_ord_equiv_with_eq() {
    // PartialOrd must be consistent with Eq
    let a = bs("10101");
    let b = bs("10101");
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
    assert_eq!(a, b);
}

#[test]
fn attack_hash_consistency() {
    // Equal bit strings must have equal hashes
    let a = bs("1010101");
    let b = bs("1010101");
    assert_eq!(hash(&a), hash(&b));

    // Different bit strings should probably have different hashes
    // (not guaranteed, but if they collide for simple cases it's suspicious)
    let c = bs("1010100");
    assert_ne!(hash(&a), hash(&c));

    // Empty
    let e1 = BitString::new();
    let e2 = BitString::new();
    assert_eq!(hash(&e1), hash(&e2));
}

#[test]
fn attack_hash_vs_eq() {
    // If two values are equal, they must hash the same
    for len in [0, 1, 5, 63, 64, 65, 128, 200] {
        let a = BitString::ones(len);
        let b = BitString::ones(len);
        assert_eq!(a, b);
        assert_eq!(hash(&a), hash(&b));

        // Construct the same value differently
        let mut c = BitString::new();
        for _ in 0..len {
            c.push(true);
        }
        assert_eq!(a, c);
        assert_eq!(hash(&a), hash(&c));
    }
}
