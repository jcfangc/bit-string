use super::*;

#[test]
fn attack_push_pop_cycle() {
    let mut bits = BitString::new();
    // Push bits on and off repeatedly
    for _ in 0..10_000 {
        bits.push(true);
        bits.push(false);
        assert_eq!(bits.pop(), Some(false));
        assert_eq!(bits.pop(), Some(true));
    }
    assert!(bits.is_empty());
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_pop_empty() {
    let mut bits = BitString::new();
    assert_eq!(bits.pop(), None);
    assert_eq!(bits.pop(), None);
}

#[test]
fn attack_push_many_word_boundaries() {
    let mut bits = BitString::new();
    for i in 0..257 {
        bits.push(i % 2 == 0);
    }
    assert_eq!(bits.bit_len(), 257);
    assert!(view_has_same_invariants(&bits));

    // Verify all bits
    for i in 0..257 {
        assert_eq!(bits.get(i), Some(i % 2 == 0), "wrong at index {i}");
    }
}

#[test]
fn attack_insert_at_boundaries() {
    let mut bits = BitString::new();

    // Insert into empty at index 0
    bits.insert(0, true);
    assert_eq!(bits.bit_len(), 1);
    assert_eq!(bits.get(0), Some(true));

    // Insert at the end (index = len)
    bits.insert(1, false);
    assert_eq!(bits.bit_len(), 2);
    assert_eq!(bits.to_string(), "10");

    // Insert at the beginning
    bits.insert(0, true);
    assert_eq!(bits.to_string(), "110");

    // Insert with index beyond len (clamped)
    bits.insert(usize::MAX, false);
    assert_eq!(bits.to_string(), "1100");

    // Insert at middle
    bits.insert(2, true);
    assert_eq!(bits.to_string(), "11100");

    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_insert_word_boundaries() {
    for len in [63, 64, 65, 127, 128, 129] {
        let base = BitString::zeros(len);
        for insert_at in [0, len / 2, len] {
            let mut bits = base.clone();
            bits.insert(insert_at, true);
            assert_eq!(
                bits.bit_len(),
                len + 1,
                "len wrong at len={len}, at={insert_at}"
            );
            assert!(view_has_same_invariants(&bits));
            assert_eq!(bits.get(insert_at), Some(true));

            // Verify prefix unchanged
            for i in 0..insert_at {
                assert_eq!(
                    bits.get(i),
                    Some(false),
                    "prefix at {i} corrupted, len={len}, at={insert_at}"
                );
            }
            // Verify suffix shifted
            for i in insert_at + 1..bits.bit_len() {
                assert_eq!(
                    bits.get(i),
                    Some(false),
                    "suffix at {i} corrupted, len={len}, at={insert_at}"
                );
            }
        }
    }
}

#[test]
fn attack_remove_boundaries() {
    // Remove from empty
    let mut bits = BitString::new();
    assert!(!bits.remove(0));
    assert!(!bits.remove(usize::MAX));

    // Remove at boundaries
    let mut bits = bs("10110"); // len 5
    assert!(!bits.remove(5)); // index == len => no-op
    assert!(!bits.remove(usize::MAX)); // beyond len => no-op
    assert!(view_has_same_invariants(&bits));
    assert_eq!(bits.to_string(), "10110");
}

#[test]
fn attack_remove_word_boundaries() {
    for len in [64, 65, 128, 129] {
        let ones = BitString::ones(len);
        for remove_at in [0, len - 1, len / 2] {
            let mut bits = ones.clone();
            assert!(
                bits.remove(remove_at),
                "remove at {remove_at} should return true for ones"
            );
            assert_eq!(bits.bit_len(), len - 1);
            assert!(view_has_same_invariants(&bits));
            assert_eq!(bits.count_ones(), len - 1);

            // Verify all remaining bits are 1
            for i in 0..bits.bit_len() {
                assert_eq!(
                    bits.get(i),
                    Some(true),
                    "bit {i} wrong after remove at {remove_at}, len={len}"
                );
            }
        }
    }
}

#[test]
fn attack_set_out_of_bounds() {
    let mut bits = bs("101");
    assert_eq!(bits.set(3, true), None);
    assert_eq!(bits.set(usize::MAX, false), None);
    // Verify unchanged
    assert_eq!(bits.to_string(), "101");
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_set_chunk_cross_word() {
    // Write that spans two words
    let mut bits = BitString::zeros(128);
    // Write 64 bits starting at bit 32: spans [32, 96)
    bits.set_chunk(32, u64::MAX, 64);
    assert!(view_has_same_invariants(&bits));

    // First 32 bits should be 0
    for i in 0..32 {
        assert_eq!(bits.get(i), Some(false), "bit {i} should be 0");
    }
    // Bits 32..96 should be 1
    for i in 32..96 {
        assert_eq!(bits.get(i), Some(true), "bit {i} should be 1");
    }
    // Bits 96..128 should be 0
    for i in 96..128 {
        assert_eq!(bits.get(i), Some(false), "bit {i} should be 0");
    }
}

#[test]
#[ignore = "BUG: set_chunk doesn't mask unused high bits after writing (marker: B1)"]
fn attack_set_chunk_beyond_len() {
    let mut bits = bs("111");
    // Write beyond the length — should be a no-op for the out-of-range part
    bits.set_chunk(3, u64::MAX, 64);
    assert_eq!(bits.to_string(), "111");
    assert!(view_has_same_invariants(&bits));

    // Partially beyond
    bits.set_chunk(2, u64::MAX, 64);
    // Only bit 2 should be in range
    assert_eq!(bits.get(2), Some(true));
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_set_chunk_len_overflow() {
    // len > 64 — low_mask(len) should mask to just the low bits
    let mut bits = BitString::zeros(64);
    bits.set_chunk(0, u64::MAX, 100); // len 100, but only 64 bits fit
    // With mask, all bits set
    assert_eq!(bits.to_string(), "1".repeat(64));
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_truncate_clamping() {
    let mut bits = bs("10101");
    bits.truncate(10); // larger => no-op
    assert_eq!(bits.to_string(), "10101");

    bits.truncate(usize::MAX); // still no-op
    assert_eq!(bits.to_string(), "10101");

    bits.truncate(0); // full truncation
    assert!(bits.is_empty());
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_truncate_word_boundaries() {
    for len in [64, 65, 128, 129] {
        for new_len in [0, 1, 63, 64, 65] {
            if new_len > len {
                continue;
            }
            let mut bits = BitString::ones(len);
            bits.truncate(new_len);
            assert_eq!(bits.bit_len(), new_len, "len={len} new_len={new_len}");
            assert!(
                view_has_same_invariants(&bits),
                "invariant broken: len={len} new_len={new_len}"
            );
            assert_eq!(bits.count_ones(), new_len);
        }
    }
}

#[test]
fn attack_clear() {
    let mut bits = BitString::ones(128);
    bits.clear();
    assert!(bits.is_empty());
    assert!(bits.words().is_empty());
    assert!(view_has_same_invariants(&bits));
}
