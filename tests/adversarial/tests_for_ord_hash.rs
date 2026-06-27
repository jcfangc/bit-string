use super::*;
use core::cmp::Ordering;
use int_interval::UsizeCO;

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

// ===========================================================================
// A. Both-sides-unaligned Eq / Ord / Hash
// ===========================================================================

#[test]
fn attack_eq_both_unaligned_same_bits() {
    // "0"*10 + "1111111111" + "0"*10 = 30 bits.  The middle ones block is at [10,20).
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
    ]));
    let b = a.clone();

    // Both sub-views at different unaligned offsets, both within the ones block.
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 8).unwrap()); // "11111111"
    let vb = b
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(12, 8).unwrap()); // "11111111"
    assert_eq!(va, vb);
    assert_eq!(va.to_bit_string(), vb.to_bit_string());
}

#[test]
fn attack_eq_both_unaligned_different_bits() {
    // "1"*20 + "0"*20.  Offset 1 -> "1111", offset 19 -> "1000".
    let a = bs(&cat(&["1".repeat(20).as_str(), "0".repeat(20).as_str()]));
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(1, 4).unwrap()); // "1111"
    let vb = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(19, 4).unwrap()); // "1000"
    assert_ne!(va, vb);
}

#[test]
fn attack_ord_both_unaligned() {
    // "1"*30 + "0"*30.   "11" vs "10" from different unaligned offsets.
    let a = bs(&cat(&["1".repeat(30).as_str(), "0".repeat(30).as_str()]));
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(3, 2).unwrap()); // "11"
    let vb = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(29, 2).unwrap()); // "10"
    assert!(va > vb);
    assert_eq!(va.partial_cmp(&vb), Some(Ordering::Greater));
}

#[test]
fn attack_ord_both_unaligned_equal() {
    // "0"*5 + "111111" + "0"*5 = 16 bits.  "1111" at offsets 5 and 7.
    let a = bs(&cat(&[
        "0".repeat(5).as_str(),
        "111111",
        "0".repeat(5).as_str(),
    ]));
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(5, 4).unwrap()); // "1111"
    let vb = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(7, 4).unwrap()); // "1111"
    assert_eq!(va.cmp(&vb), Ordering::Equal);
}

#[test]
fn attack_hash_both_unaligned_same_bits() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
    ]));
    let b = a.clone();
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 8).unwrap());
    let vb = b
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(12, 8).unwrap());
    assert_eq!(va, vb);
    assert_eq!(hash(&va), hash(&vb));
}

#[test]
fn attack_hash_unaligned_vs_aligned() {
    let a = bs(&cat(&[
        "0".repeat(20).as_str(),
        "11001100",
        "0".repeat(20).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(20, 8).unwrap());
    let owned = bs("11001100");
    assert_eq!(view, owned.as_bit_str());
    assert_eq!(hash(&view), hash(&owned.as_bit_str()));
}

// ===========================================================================
// C. Structural vs semantic Eq
// ===========================================================================

#[test]
fn attack_structural_eq_agrees_with_semantic_eq() {
    for len in [0, 1, 63, 64, 65, 128] {
        let a = BitString::ones(len);
        let b = BitString::ones(len);
        assert_eq!(a, b);
        assert_eq!(a.as_bit_str(), b.as_bit_str());

        let mut c = a.clone();
        if len > 2 {
            c.set(1, false);
        }
        let d = c.clone();
        assert_eq!(c, d);
        assert_eq!(c.as_bit_str(), d.as_bit_str());
    }
}

#[test]
fn attack_structural_eq_with_extra_capacity() {
    let mut a = BitString::zeros(10);
    a.push_bit_string(&BitString::zeros(100));
    a.truncate(10);
    let b = BitString::zeros(10);
    assert_eq!(a, b);
    assert_eq!(a.as_bit_str(), b.as_bit_str());
}
