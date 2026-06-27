use super::*;

#[test]
fn attack_extend_empty() {
    let mut bits = BitString::new();
    bits.extend(std::iter::empty::<bool>());
    assert!(bits.is_empty());

    bits.extend([true, false, true]);
    assert_eq!(bits.to_string(), "101");
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_extend_large() {
    let mut bits = BitString::new();
    let many: Vec<bool> = (0..10000).map(|i| i % 2 == 0).collect();
    bits.extend(many.iter());
    assert_eq!(bits.bit_len(), 10000);
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_exact_size_iter_collect() {
    let bits: BitString = (0..200).map(|i| i % 2 == 0).collect();
    assert_eq!(bits.bit_len(), 200);
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_stress_mixed_operations() {
    let mut bits = BitString::new();

    // Random-ish sequence of operations
    for i in 0..1000 {
        bits.push(i % 3 == 0);
    }
    assert!(view_has_same_invariants(&bits));

    // Remove every third bit
    let mut i = 0;
    while i < bits.bit_len() {
        if bits.get(i) == Some(true) {
            bits.remove(i);
        } else {
            i += 1;
        }
    }
    assert!(view_has_same_invariants(&bits));

    // All remaining should be false
    assert!(bits.is_all_zeros());

    // Flip all
    bits.not_assign();
    assert!(bits.is_all_ones());
    assert!(view_has_same_invariants(&bits));

    // Shift
    bits.shl_assign(1);
    assert_eq!(bits.get(0), Some(false));
    assert!(view_has_same_invariants(&bits));

    // Truncate
    bits.truncate(64);
    assert_eq!(bits.bit_len(), 64);
    assert!(view_has_same_invariants(&bits));

    // Split and re-merge
    let tail = bits.split_off(32);
    bits.push_bit_string(&tail);
    assert_eq!(bits.bit_len(), 64);
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_large_allocation() {
    let len = 1_000_000;
    let bits = BitString::zeros(len);
    assert_eq!(bits.bit_len(), len);
    assert!(view_has_same_invariants(&bits));
    assert_eq!(bits.count_ones(), 0);
}

#[test]
fn attack_repeated_growth_shrink() {
    let mut bits = BitString::new();
    for _ in 0..100 {
        for _ in 0..100 {
            bits.push(true);
        }
        assert!(view_has_same_invariants(&bits));
        for _ in 0..50 {
            bits.pop();
        }
        assert!(view_has_same_invariants(&bits));
    }
}

#[test]
fn attack_set_chunk_exact_boundaries() {
    // Test every combination of start offset and length around word boundary
    for start in [0, 1, 31, 32, 33, 62, 63, 64, 65, 95, 96, 100, 127, 128] {
        for len in [1, 8, 16, 32, 33, 63] {
            let mut bits = BitString::zeros(256);
            let pattern: u64 = 0xAAAA_AAAA_AAAA_AAAA;
            bits.set_chunk(start, pattern, len);

            let mask = if len >= 64 {
                u64::MAX
            } else {
                (1u64 << len).wrapping_sub(1)
            };
            let expected_mask = pattern & mask;
            let chunk = bits.get_chunk(start);
            assert_eq!(
                chunk & mask,
                expected_mask,
                "set_chunk/get_chunk mismatch at start={start}, len={len}"
            );
        }
    }
}

#[test]
fn attack_mask_invariant_after_every_operation() {
    // After every mutation, the last word must have unused bits zeroed
    let mut bits = bs("1111111111");

    bits.push(true);
    assert!(view_has_same_invariants(&bits));
    bits.pop();
    assert!(view_has_same_invariants(&bits));
    bits.insert(3, false);
    assert!(view_has_same_invariants(&bits));
    bits.remove(3);
    assert!(view_has_same_invariants(&bits));
    bits.set(0, false);
    assert!(view_has_same_invariants(&bits));
    bits.truncate(5);
    assert!(view_has_same_invariants(&bits));
    bits.shl_assign(2);
    assert!(view_has_same_invariants(&bits));
    bits.shr_assign(1);
    assert!(view_has_same_invariants(&bits));
    bits.not_assign();
    assert!(view_has_same_invariants(&bits));
}
