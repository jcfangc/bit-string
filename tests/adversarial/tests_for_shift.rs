use super::*;

#[test]
fn attack_shift_zero() {
    let bits = bs("10101");
    assert_eq!(bits.shl(0), bits);
    assert_eq!(bits.shr(0), bits);

    let mut bits2 = bits.clone();
    bits2.shl_assign(0);
    assert_eq!(bits2, bits);

    let mut bits3 = bits.clone();
    bits3.shr_assign(0);
    assert_eq!(bits3, bits);
}

#[test]
fn attack_shift_beyond_len() {
    let bits = bs("10101"); // len 5

    let left = bits.shl(5);
    assert_eq!(left, BitString::zeros(5));
    let left = bits.shl(100);
    assert_eq!(left, BitString::zeros(5));
    let left = bits.shl(usize::MAX);
    assert_eq!(left, BitString::zeros(5));

    let right = bits.shr(5);
    assert_eq!(right, BitString::zeros(5));
    let right = bits.shr(100);
    assert_eq!(right, BitString::zeros(5));
    let right = bits.shr(usize::MAX);
    assert_eq!(right, BitString::zeros(5));
}

#[test]
fn attack_shift_word_boundaries() {
    for len in [64, 65, 128, 129] {
        let ones = BitString::ones(len);
        for amount in [1, 32, 63, 64, 65, 127, 128, len] {
            let left = ones.shl(amount);
            assert_eq!(left.bit_len(), len);
            assert!(view_has_same_invariants(&left));
            // First `amount` bits should be 0
            for i in 0..amount.min(len) {
                assert_eq!(
                    left.get(i),
                    Some(false),
                    "shl: bit {i} should be 0, len={len}, amt={amount}"
                );
            }
            // Remaining should be 1
            for i in amount..len {
                assert_eq!(
                    left.get(i),
                    Some(true),
                    "shl: bit {i} should be 1, len={len}, amt={amount}"
                );
            }

            let right = ones.shr(amount);
            assert_eq!(right.bit_len(), len);
            assert!(view_has_same_invariants(&right));
            // Last `amount` bits should be 0
            let from = len.saturating_sub(amount);
            for i in from..len {
                assert_eq!(
                    right.get(i),
                    Some(false),
                    "shr: bit {i} should be 0, len={len}, amt={amount}"
                );
            }
            // Leading bits should be 1
            for i in 0..from {
                assert_eq!(
                    right.get(i),
                    Some(true),
                    "shr: bit {i} should be 1, len={len}, amt={amount}"
                );
            }
        }
    }
}

#[test]
fn attack_shift_assign_mutate() {
    let mut bits = bs("10101");
    bits.shl_assign(2);
    assert_eq!(bits.to_string(), "00101");
    assert!(view_has_same_invariants(&bits));

    let mut bits = bs("10101");
    bits.shr_assign(2);
    assert_eq!(bits.to_string(), "10100");
    assert!(view_has_same_invariants(&bits));
    assert!(view_has_same_invariants(&bits));
}
