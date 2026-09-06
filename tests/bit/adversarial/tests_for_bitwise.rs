use super::*;

#[test]
fn attack_not_empty() {
    let bits = BitString::new();
    let neg = bits.not();
    assert!(neg.is_empty());

    let bits = BitString::zeros(64);
    let neg = bits.not();
    assert_eq!(neg.to_string(), "1".repeat(64));

    let bits = BitString::ones(65);
    let neg = bits.not();
    assert_eq!(neg.to_string(), "0".repeat(65));
    assert!(view_has_same_invariants(&neg));
}

#[test]
fn attack_not_assign_idempotent() {
    let mut bits = bs("10101");
    bits.not_assign();
    bits.not_assign();
    assert_eq!(bits.to_string(), "10101");
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_binary_ops_length_mismatch() {
    let a = bs("101");
    let b = bs("1010");

    assert!(a.and(&b).is_err());
    assert!(a.or(&b).is_err());
    assert!(a.xor(&b).is_err());

    let mut a = a.clone();
    assert!(a.and_assign(&b).is_err());
    assert!(a.or_assign(&b).is_err());
    assert!(a.xor_assign(&b).is_err());
}

#[test]
fn attack_binary_ops_identity() {
    let a = bs("10101");

    // AND with ones = identity
    assert_eq!(a.and(&BitString::ones(5)).unwrap(), a);

    // OR with zeros = identity
    assert_eq!(a.or(&BitString::zeros(5)).unwrap(), a);

    // XOR with zeros = identity
    assert_eq!(a.xor(&BitString::zeros(5)).unwrap(), a);

    // XOR with self = zeros
    assert_eq!(a.xor(&a).unwrap(), BitString::zeros(5));

    // AND with zeros = zeros
    assert_eq!(a.and(&BitString::zeros(5)).unwrap(), BitString::zeros(5));
}

#[test]
fn attack_binary_ops_invariants() {
    for len in [0, 1, 63, 64, 65, 127, 128, 129] {
        let a = BitString::ones(len);
        let b = BitString::zeros(len);

        let r = a.and(&b).unwrap();
        assert!(view_has_same_invariants(&r));

        let r = a.or(&b).unwrap();
        assert!(view_has_same_invariants(&r));

        let r = a.xor(&b).unwrap();
        assert!(view_has_same_invariants(&r));

        let r = a.not();
        assert!(view_has_same_invariants(&r));
    }
}
