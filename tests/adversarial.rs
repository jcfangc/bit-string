//! Adversarial tests — attack every public API with edge cases, fuzz-style
//! inputs, and adversarial combinations that might trigger panics, incorrect
//! results, or invariant violations.

// rustc flags for NEON cross-check
#![cfg_attr(
    all(target_arch = "aarch64", target_feature = "neon"),
    feature(stdarch_aarch64_prefetch)
)]

use bit_string::BitString;
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use int_interval::UsizeCO;
use std::collections::hash_map::DefaultHasher;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn hash<T: Hash>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

fn bs(s: &str) -> BitString {
    s.parse().unwrap()
}

fn view_has_same_invariants(bits: &BitString) -> bool {
    // Internal invariants: bit_len matches actual bit count; unused bits in
    // last word are zeroed.
    let word_count = (bits.bit_len() + 63) / 64;
    if bits.words().len() != word_count {
        return false;
    }
    if bits.bit_len() == 0 {
        return bits.words().is_empty();
    }
    let rem = bits.bit_len() % 64;
    if rem != 0 {
        let last = bits.words().last().unwrap();
        let mask = (1u64 << rem).wrapping_sub(1);
        if last & !mask != 0 {
            return false;
        }
    }
    true
}

/// Scan every bit and verify `.get()` matches direct inspection of words.
#[allow(dead_code)]
fn verify_all_bits(bits: &BitString) -> bool {
    for i in 0..bits.bit_len() {
        let word = bits.words()[i / 64];
        let expected = (word >> (i % 64)) & 1 != 0;
        if bits.get(i) != Some(expected) {
            return false;
        }
    }
    true
}

// ===========================================================================
// 1. Construction attack vectors
// ===========================================================================

#[test]
fn attack_empty_construction() {
    let a = BitString::new();
    assert!(a.is_empty());
    assert_eq!(a.bit_len(), 0);
    assert!(a.words().is_empty());
    assert!(view_has_same_invariants(&a));

    let b = BitString::default();
    assert_eq!(a, b);

    let c = BitString::zeros(0);
    assert_eq!(a, c);

    let d = BitString::ones(0);
    assert_eq!(a, d);

    let e = BitString::repeat(true, 0);
    let f = BitString::repeat(false, 0);
    assert_eq!(e, f);
}

#[test]
fn attack_from_words_dirty_high_bits() {
    // from_words should mask dirty high bits in the last word
    let words = &[u64::MAX, u64::MAX];
    let bits = BitString::from_words(words, 65).unwrap();
    assert!(view_has_same_invariants(&bits));

    // The 65th bit (bit 1 of second word) should be present, bits 66+ zeroed
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), None);

    // Verify all bits beyond len are reported as None
    for i in 65..128 {
        assert!(bits.get(i).is_none(), "bit {i} should be out of range");
    }
}

#[test]
fn attack_from_words_wrong_count() {
    // Too few words
    assert!(BitString::from_words(&[0u64], 65).is_none());
    // Too many words
    assert!(BitString::from_words(&[0u64, 0], 63).is_none());
    // Zero length with non-empty words
    assert!(BitString::from_words(&[1u64], 0).is_none());
    // Empty words with non-zero length
    assert!(BitString::from_words(&[], 1).is_none());
}

#[test]
fn attack_from_str_invalid() {
    assert!("".parse::<BitString>().is_ok()); // empty is valid
    assert!("0".parse::<BitString>().is_ok());
    assert!("1".parse::<BitString>().is_ok());
    assert!("2".parse::<BitString>().is_err());
    assert!("xyz".parse::<BitString>().is_err());
    assert!("0101x010".parse::<BitString>().is_err());
    assert!(" ".parse::<BitString>().is_err());
    assert!("\0".parse::<BitString>().is_err());
    assert!("\n".parse::<BitString>().is_err());
    // Unicode
    assert!("🔥".parse::<BitString>().is_err());
    // Very long valid string
    let s = "01".repeat(10_000);
    assert!(s.parse::<BitString>().is_ok());
}

#[test]
fn attack_from_bool_slice_empty() {
    let a = BitString::from(&[] as &[bool]);
    assert!(a.is_empty());

    let b = BitString::from([false; 0]);
    assert!(b.is_empty());

    let c: BitString = [].into_iter().collect();
    assert!(c.is_empty());
}

#[test]
fn attack_from_bool_iter_large() {
    let many: Vec<bool> = (0..100_000).map(|i| i % 3 == 0).collect();
    let bits = BitString::from_iter(many.iter().copied());
    assert_eq!(bits.bit_len(), 100_000);
    assert!(view_has_same_invariants(&bits));

    // Spot-check
    for i in [0, 3, 6, 99999] {
        let expected = i % 3 == 0;
        assert_eq!(bits.get(i), Some(expected), "mismatch at index {i}");
    }
}

#[test]
fn attack_zeros_ones_word_boundaries() {
    for len in [0, 1, 63, 64, 65, 127, 128, 129, 1023, 1024, 1025] {
        let zeros = BitString::zeros(len);
        assert!(view_has_same_invariants(&zeros));
        assert_eq!(zeros.count_ones(), 0);
        assert_eq!(zeros.bit_len(), len);

        let ones = BitString::ones(len);
        assert!(view_has_same_invariants(&ones));
        assert_eq!(ones.count_ones(), len);
        assert_eq!(ones.bit_len(), len);

        let rep = BitString::repeat(true, len);
        assert_eq!(rep, ones);
    }
}

// ===========================================================================
// 2. Access attack vectors
// ===========================================================================

#[test]
fn attack_get_out_of_bounds() {
    let bits = bs("10101"); // len 5
    assert_eq!(bits.get(0), Some(true));
    assert_eq!(bits.get(4), Some(true));
    assert_eq!(bits.get(5), None);
    assert_eq!(bits.get(usize::MAX), None);
    assert_eq!(bits.get(usize::MAX / 2), None);
}

#[test]
fn attack_first_last_empty() {
    let bits = BitString::new();
    assert_eq!(bits.first(), None);
    assert_eq!(bits.last(), None);
}

#[test]
fn attack_get_chunk_boundaries() {
    // Empty
    let empty = BitString::new();
    assert_eq!(empty.get_chunk(0), 0);
    assert_eq!(empty.get_chunk(usize::MAX), 0);

    // Single bit
    let one = bs("1");
    assert_eq!(one.get_chunk(0), 1);
    assert_eq!(one.get_chunk(1), 0);

    // Cross-word boundary: 65 bits, start at 32
    let mut bits = BitString::zeros(65);
    bits.set_chunk(32, u64::MAX, 33);
    let chunk = bits.get_chunk(32);
    // Should have 33 valid bits
    assert_eq!(chunk & ((1u64 << 33) - 1), (1u64 << 33) - 1);

    // Start beyond len: should return 0
    let bits = bs("1111");
    assert_eq!(bits.get_chunk(4), 0);
    assert_eq!(bits.get_chunk(100), 0);
}

#[test]
fn attack_read_write_consistency() {
    // Every read should match what was written
    let mut bits = BitString::zeros(256);
    for i in 0..256 {
        let val = i % 7 == 0 || i % 3 == 0;
        bits.set(i, val);
        assert_eq!(bits.get(i), Some(val));
    }
    assert!(view_has_same_invariants(&bits));
}

// ===========================================================================
// 3. Editing attack vectors
// ===========================================================================

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
    assert!(!bits.remove(5)); // index == len → no-op
    assert!(!bits.remove(usize::MAX)); // beyond len → no-op
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
    bits.truncate(10); // larger → no-op
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

// ===========================================================================
// 4. Concat / split attack vectors
// ===========================================================================

#[test]
fn attack_push_bit_string_self() {
    // Self-append should work (we clone the source first)
    let mut bits = bs("101");
    let clone = bits.clone();
    bits.push_bit_string(&clone);
    assert_eq!(bits.to_string(), "101101");
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_push_bit_string_empty() {
    let mut bits = bs("101");
    let empty = BitString::new();
    bits.push_bit_string(&empty);
    assert_eq!(bits.to_string(), "101");
    assert!(view_has_same_invariants(&bits));

    // Empty + non-empty
    let mut empty = BitString::new();
    let other = bs("11");
    empty.push_bit_string(&other);
    assert_eq!(empty.to_string(), "11");
    assert!(view_has_same_invariants(&empty));
}

#[test]
fn attack_push_bit_string_word_boundaries() {
    // Every combination of lengths around word boundaries
    for a_len in [0, 63, 64, 65] {
        for b_len in [0, 1, 63, 64, 65, 127] {
            let a = BitString::ones(a_len);
            let b = BitString::zeros(b_len);
            let mut c = a.clone();
            c.push_bit_string(&b);

            let expected_len = a_len + b_len;
            assert_eq!(c.bit_len(), expected_len, "len: {a_len}+{b_len}");

            // First a_len bits should be ones
            for i in 0..a_len {
                assert_eq!(c.get(i), Some(true), "pos {i}/{a_len}+{b_len}");
            }
            // Remaining should be zeros
            for i in a_len..expected_len {
                assert_eq!(c.get(i), Some(false), "pos {i}/{a_len}+{b_len}");
            }
            assert!(view_has_same_invariants(&c));
        }
    }
}

#[test]
fn attack_insert_bit_string_self() {
    let bits = bs("111");
    let clone = bits.clone();
    let mut result = bits.clone();
    result.insert_bit_string(1, &clone); // insert "111" at index 1
    assert_eq!(result.to_string(), "111111"); // "1" + "111" + "11" = 6 bits
    assert!(view_has_same_invariants(&result));
}

#[test]
fn attack_insert_bit_string_boundaries() {
    let mut bits = BitString::new();

    // Insert into empty at index 0
    bits.insert_bit_string(0, &bs("101"));
    assert_eq!(bits.to_string(), "101");

    // Insert at beginning
    bits.insert_bit_string(0, &bs("01"));
    assert_eq!(bits.to_string(), "01101");

    // Insert at end
    bits.insert_bit_string(usize::MAX, &bs("10"));
    assert_eq!(bits.to_string(), "0110110");

    // Insert empty string
    let len_before = bits.bit_len();
    bits.insert_bit_string(2, &BitString::new());
    assert_eq!(bits.bit_len(), len_before);

    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_insert_bit_string_word_boundaries() {
    let filler = BitString::ones(40);
    for host_len in [63, 64, 65] {
        let host = BitString::zeros(host_len);
        for at in [0, host_len / 2, host_len] {
            let mut bits = host.clone();
            bits.insert_bit_string(at, &filler);
            assert_eq!(bits.bit_len(), host_len + 40);
            assert!(view_has_same_invariants(&bits));

            // 40 ones should be at `at`
            for i in at..at + 40 {
                assert_eq!(
                    bits.get(i),
                    Some(true),
                    "bit {i} should be 1, host_len={host_len}, at={at}"
                );
            }
        }
    }
}

#[test]
fn attack_split_off_boundaries() {
    let mut bits = bs("1010101"); // len 7

    // Split at len
    let tail = bits.split_off(7);
    assert!(tail.is_empty());
    assert_eq!(bits.to_string(), "1010101");

    // Split at usize::MAX (clamped to 7)
    let tail = bits.split_off(usize::MAX);
    assert!(tail.is_empty());
    assert_eq!(bits.to_string(), "1010101");

    // Split at 0
    let tail = bits.split_off(0);
    assert_eq!(tail.to_string(), "1010101");
    assert!(bits.is_empty());
    assert!(view_has_same_invariants(&tail));

    bits = bs("1010101");
    // Split in middle
    let tail = bits.split_off(3);
    assert_eq!(bits.to_string(), "101");
    assert_eq!(tail.to_string(), "0101");
    assert!(view_has_same_invariants(&bits));
    assert!(view_has_same_invariants(&tail));
}

#[test]
fn attack_split_off_word_boundaries() {
    for len in [63, 64, 65, 127, 128, 129] {
        let mut bits = BitString::ones(len);
        for at in [0, 1, 32, 63, 64, 65, len] {
            if at > len {
                continue;
            }
            let tail = bits.split_off(at);
            assert_eq!(bits.bit_len(), at, "len={len}, at={at}");
            assert_eq!(tail.bit_len(), len - at, "len={len}, at={at}");
            assert!(view_has_same_invariants(&bits));
            assert!(view_has_same_invariants(&tail));

            // Reassemble and verify
            bits.push_bit_string(&tail);
            assert_eq!(bits.bit_len(), len);
            assert_eq!(bits.count_ones(), len);

            // Reset for next iteration
            bits = BitString::ones(len);
        }
    }
}

// ===========================================================================
// 5. Slice / drain / replace attack vectors
// ===========================================================================

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

#[test]
fn attack_replace_equal_length() {
    // Replace with same-length — should be in-place
    let mut bits = bs("11110000");
    let repl = bs("0011");
    bits.replace_assign(2, &repl);
    assert_eq!(bits.to_string(), "11001100");
    assert!(view_has_same_invariants(&bits));

    // Replace via interval
    let bits = bs("11110000");
    let result = bits.replace_interval(UsizeCO::checked_from_start_len(2, 4).unwrap(), &repl);
    assert_eq!(result.to_string(), "11001100");
    assert!(view_has_same_invariants(&result));
}

#[test]
fn attack_replace_different_lengths() {
    // Replace with shorter — replace 1 bit at index 2 with "0"
    // Original: "11110000" → replace bit 2 (which is '1') with '0'
    // Result: "11010000"
    let mut bits = bs("11110000");
    let repl = bs("0");
    bits.replace_assign(2, &repl);
    assert_eq!(bits.to_string(), "11010000");
    assert!(view_has_same_invariants(&bits));

    // Replace with longer (clamped range < replacement length)
    let mut bits = bs("11000");
    let repl = bs("111");
    bits.replace_assign(4, &repl); // clamped range [4,5) = 1 bit, repl = 3 bits → grow
    assert_eq!(bits.to_string(), "1100111");
    assert!(view_has_same_invariants(&bits));

    // Replace with empty replacement at edge → effectively insert
    let mut bits = bs("10101");
    bits.replace_assign(0, &BitString::new());
    assert_eq!(bits.to_string(), "10101"); // empty replacement, no-op
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_replace_at_word_boundaries() {
    for len in [63, 64, 65] {
        let host = BitString::ones(len);
        let repl = BitString::zeros(5);
        for at in [0, len / 2, len.saturating_sub(5)] {
            let result = host.replace(at, &repl);
            assert_eq!(result.bit_len(), len); // same length
            assert!(view_has_same_invariants(&result));

            // Check the zeros are at the right place
            for i in at..at + 5 {
                assert_eq!(
                    result.get(i),
                    Some(false),
                    "should be 0 at {i}, len={len}, at={at}"
                );
            }
        }
    }
}

#[test]
fn attack_replace_self_as_replacement() {
    // Replace using self as replacement (should clone internally)
    let bits = bs("110011");
    let result = bits.replace_interval(UsizeCO::checked_from_start_len(2, 2).unwrap(), &bits);
    // Removed "00" (positions 2-3), inserted "110011"
    assert_eq!(result.to_string(), "1111001111");
    assert!(view_has_same_invariants(&result));
}

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
// 6. Bitwise operation attack vectors
// ===========================================================================

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

// ===========================================================================
// 7. Shift attack vectors
// ===========================================================================

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

// ===========================================================================
// 8. Count / leading-trailing attack vectors
// ===========================================================================

#[test]
fn attack_count_ones_zeros() {
    for len in [0, 1, 63, 64, 65, 127, 128, 129] {
        let zeros = BitString::zeros(len);
        assert_eq!(zeros.count_ones(), 0);
        assert_eq!(zeros.count_zeros(), len);

        let ones = BitString::ones(len);
        assert_eq!(ones.count_ones(), len);
        assert_eq!(ones.count_zeros(), 0);

        let _mixed = BitString::repeat(true, len);
        let _ = _mixed; // placeholder
    }
}

#[test]
fn attack_leading_zeros_word_boundaries() {
    // Leading zeros on word-aligned views
    for len in [0, 1, 64, 65, 128, 129] {
        let bits = BitString::zeros(len);
        assert_eq!(
            bits.leading_zeros(),
            len,
            "leading_zeros on zeros len={len}"
        );
        assert_eq!(bits.leading_ones(), 0, "leading_ones on zeros len={len}");
        assert_eq!(
            bits.trailing_zeros(),
            len,
            "trailing_zeros on zeros len={len}"
        );
        assert_eq!(bits.trailing_ones(), 0, "trailing_ones on zeros len={len}");

        let bits = BitString::ones(len);
        assert_eq!(bits.leading_zeros(), 0);
        assert_eq!(bits.leading_ones(), len);
        assert_eq!(bits.trailing_zeros(), 0);
        assert_eq!(bits.trailing_ones(), len);
    }
}

#[test]
fn attack_leading_zeros_mixed() {
    // "0011100" — 2 leading zeros, 1 trailing? No, 2 trailing zeros
    let bits = bs("0011100");
    assert_eq!(bits.leading_zeros(), 2);
    assert_eq!(bits.leading_ones(), 0);
    assert_eq!(bits.trailing_zeros(), 2);
    assert_eq!(bits.trailing_ones(), 0);

    // "1110001" — 3 leading ones, 0 trailing zeros? No, 1 trailing one (since end is "1")
    // Wait: "1110001" = positions: 0=1, 1=1, 2=1, 3=0, 4=0, 5=0, 6=1
    let bits = bs("1110001");
    assert_eq!(bits.leading_zeros(), 0);
    assert_eq!(bits.leading_ones(), 3);
    assert_eq!(bits.trailing_zeros(), 0);
    assert_eq!(bits.trailing_ones(), 1);

    // Single bit
    let bits = bs("0");
    assert_eq!(bits.leading_zeros(), 1);
    assert_eq!(bits.trailing_zeros(), 1);

    let bits = bs("1");
    assert_eq!(bits.leading_ones(), 1);
    assert_eq!(bits.trailing_ones(), 1);
}

#[test]
fn attack_bitstr_view_leading_trailing() {
    // Create a bitstring, take a sub-view, check leading/trailing on the view
    let bits = bs("000111000");
    let view = bits.as_bit_str();

    // view of "00111" at offset 1, len 5
    let sub = view.slice(UsizeCO::checked_from_start_len(1, 5).unwrap());
    assert_eq!(sub.to_bit_string().to_string(), "00111");
    assert_eq!(sub.leading_zeros(), 2);
    assert_eq!(sub.leading_ones(), 0);
    assert_eq!(sub.trailing_zeros(), 0);
    assert_eq!(sub.trailing_ones(), 3);

    // Misaligned start within a word: offset 3, len 5 from the original
    // "11100" from bits "000111000" at offset 3
    let sub2 = view.slice(UsizeCO::checked_from_start_len(3, 5).unwrap());
    assert_eq!(sub2.to_bit_string().to_string(), "11100");
    assert_eq!(sub2.leading_zeros(), 0);
    assert_eq!(sub2.leading_ones(), 3);
    assert_eq!(sub2.trailing_zeros(), 2);
    assert_eq!(sub2.trailing_ones(), 0);
}

#[test]
#[ignore = "BUG: BitStr::count_ones doesn't mask last partial word (marker: B2)"]
fn attack_bitstr_count_ones_misaligned() {
    // Test count_ones on BitStr views with unaligned starts
    let bits = bs(&("0".repeat(200) + "1" + &"0".repeat(63)));
    let view = bits.as_bit_str();

    // Slice that starts and ends at various offsets
    for start in 0..200 {
        for len in 1..64.min(264 - start) {
            let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
            let sub = view.slice(interval);
            let owned = sub.to_bit_string();
            assert_eq!(
                sub.count_ones(),
                owned.count_ones(),
                "count_ones mismatch at start={start}, len={len}"
            );
            assert_eq!(
                sub.count_zeros(),
                owned.count_zeros(),
                "count_zeros mismatch at start={start}, len={len}"
            );
        }
    }
}

#[test]
#[ignore = "BUG: BitStr::count_ones doesn't mask last partial word (marker: B2)"]
fn attack_count_ones_at_all_offsets() {
    // Systematic test: pattern "010101..." at every offset for every length
    let pattern = "01".repeat(10); // 20 bits
    let bits = bs(&pattern);
    let view = bits.as_bit_str();

    for start in 0..20 {
        for len in 1..=20 - start {
            let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
            let sub = view.slice(interval);
            let owned = sub.to_bit_string();
            assert_eq!(
                sub.count_ones(),
                owned.count_ones(),
                "count_ones mismatch at start={start}, len={len}, pattern={pattern}"
            );
        }
    }
}

// ===========================================================================
// 9. Matching attack vectors
// ===========================================================================

#[test]
fn attack_matches_at_oob() {
    let bits = bs("10101");
    let pattern = bits.as_bit_str();

    // index beyond len
    assert!(!bits.matches_at(5, pattern));
    assert!(!bits.matches_at(usize::MAX, pattern));

    // pattern longer than remaining
    let binding = bs("101010");
    let pattern = binding.as_bit_str(); // len 6
    assert!(!bits.matches_at(0, pattern));

    // Empty pattern always matches
    let empty_binding = BitString::new();
    let empty = empty_binding.as_bit_str();
    assert!(bits.matches_at(0, empty));
    assert!(bits.matches_at(5, empty));
}

#[test]
fn attack_starts_with_ends_with_edge() {
    let bits = bs("10101");

    // Empty always matches
    assert!(bits.starts_with(BitString::new().as_bit_str()));
    assert!(bits.ends_with(BitString::new().as_bit_str()));

    // Longer than self
    assert!(!bits.starts_with(bs("101010").as_bit_str()));
    assert!(!bits.ends_with(bs("101010").as_bit_str()));

    // Exact match
    assert!(bits.starts_with(bits.as_bit_str()));
    assert!(bits.ends_with(bits.as_bit_str()));
}

#[test]
fn attack_find_empty_needle() {
    let bits = bs("10101");
    let empty_binding2 = BitString::new();
    let needle = empty_binding2.as_bit_str();

    assert_eq!(bits.find(needle), Some(0));
    assert_eq!(bits.rfind(needle), Some(5)); // rfind with empty returns len
}

#[test]
fn attack_find_needle_longer_than_haystack() {
    let bits = bs("10");
    let binding3 = bs("101");
    let needle = binding3.as_bit_str();
    assert_eq!(bits.find(needle), None);
    assert_eq!(bits.rfind(needle), None);
    assert!(!bits.contains(needle));
}

#[test]
fn attack_find_at_word_boundaries() {
    // "0" * 128 + "101" + "0" * 128
    let mut bits = BitString::zeros(128);
    bits.push_bit_string(&bs("101"));
    bits.push_bit_string(&BitString::zeros(128));
    let binding4 = bs("101");
    let needle = binding4.as_bit_str();

    assert_eq!(bits.find(needle), Some(128));
    assert_eq!(bits.rfind(needle), Some(128));
    assert!(bits.contains(needle));
}

#[test]
fn attack_find_pattern_at_word_edge() {
    // Pattern that straddles word boundary
    for offset in [32, 60, 61, 62, 63, 64, 65, 66, 67, 95, 96, 100] {
        let mut bits = BitString::zeros(256);
        let pattern = bs("110011");
        bits.replace_assign(offset, &pattern);

        let found = bits.find(pattern.as_bit_str());
        assert_eq!(found, Some(offset), "find failed at offset {offset}");

        let rfound = bits.rfind(pattern.as_bit_str());
        assert_eq!(rfound, Some(offset), "rfind failed at offset {offset}");
    }
}

#[test]
#[ignore = "BUG: BitStr::find misses occurrences straddling unaligned→aligned word boundary (marker: B3)"]
fn attack_find_multiple_occurrences() {
    let bits = bs(&("101".to_owned() + &"0".repeat(60) + "101" + &"0".repeat(60) + "101"));
    let binding5 = bs("101");
    let needle = binding5.as_bit_str();

    assert_eq!(bits.find(needle), Some(0));
    assert_eq!(bits.rfind(needle), Some(126)); // 0 + 3 + 60 + 3 + 60 = 126

    // Find all occurrences
    let mut pos = 0;
    let mut count = 0;
    while let Some(p) = bits.as_bit_str().slice_from(pos).find(needle) {
        count += 1;
        pos += p + needle.bit_len();
    }
    assert_eq!(count, 3);
}

#[test]
fn attack_strip_prefix_suffix() {
    let bits = bs("10101");

    // Strip empty
    assert_eq!(
        bits.strip_prefix(BitString::new().as_bit_str()).unwrap(),
        bits
    );
    assert_eq!(
        bits.strip_suffix(BitString::new().as_bit_str()).unwrap(),
        bits
    );

    // Mismatch
    assert!(bits.strip_prefix(bs("0").as_bit_str()).is_none());
    assert!(bits.strip_suffix(bs("0").as_bit_str()).is_none());

    // Valid strip
    let stripped = bits.strip_prefix(bs("10").as_bit_str()).unwrap();
    assert_eq!(stripped.to_string(), "101");

    let stripped = bits.strip_suffix(bs("01").as_bit_str()).unwrap();
    assert_eq!(stripped.to_string(), "101");
}

// ===========================================================================
// 10. Comparison / Hash attack vectors
// ===========================================================================

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
// 11. Display / Debug round-trip
// ===========================================================================

#[test]
fn attack_display_roundtrip() {
    for s in ["", "0", "1", "01", "10", "10101", "000", "111"] {
        let bits: BitString = s.parse().unwrap();
        assert_eq!(bits.to_string(), s, "roundtrip failed for '{s}'");
    }
}

#[test]
fn attack_display_debug_consistency() {
    let bits = bs("10101");
    let debug = format!("{:?}", bits);
    assert!(debug.contains("10101"));
    assert!(debug.starts_with("BitString("));
}

#[test]
fn attack_display_empty() {
    let bits = BitString::new();
    assert_eq!(bits.to_string(), "");
    assert_eq!(format!("{:?}", bits), "BitString(\"\")");
}

// ===========================================================================
// 12. Iterator attack vectors
// ===========================================================================

#[test]
fn attack_iter_empty() {
    let bits = BitString::new();
    let iter = bits.iter();
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.count(), 0);
    assert!(bits.to_bool_vec().is_empty());
}

#[test]
fn attack_iter_double_ended() {
    let bits = bs("10101");
    let mut iter = bits.iter();

    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(true));
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn attack_iter_exact_size() {
    for len in [0, 1, 64, 65, 128] {
        let bits = BitString::ones(len);
        let iter = bits.iter();
        assert_eq!(iter.len(), len);
        assert_eq!(iter.count(), len);
    }
}

#[test]
fn attack_iter_consistency_with_get() {
    let bits = bs("1100101");
    for (i, bit) in bits.iter().enumerate() {
        assert_eq!(Some(bit), bits.get(i));
    }
}

// ===========================================================================
// 13. Clone / invariants
// ===========================================================================

#[test]
fn attack_clone_invariants() {
    for len in [0, 1, 63, 64, 65, 127, 128, 129] {
        let bits = BitString::ones(len);
        let clone = bits.clone();
        assert_eq!(bits, clone);
        assert!(view_has_same_invariants(&clone));

        // Clone should be independent
        if len > 0 {
            let mut clone2 = bits.clone();
            clone2.set(0, false).unwrap();
            assert_ne!(bits, clone2);
        }
    }
}

// ===========================================================================
// 14. BitStr lifetime / use-after-free attempts
// ===========================================================================

#[test]
fn attack_bitstr_to_bit_string_word_boundaries() {
    // to_bit_string on sub-views at various offsets
    let bits = BitString::ones(200);
    let view = bits.as_bit_str();

    for start in [0, 1, 32, 63, 64, 65, 100, 127, 128, 129, 200] {
        for len in [1, 63, 64, 65] {
            if start + len > 200 {
                continue;
            }
            let sub = view.slice(UsizeCO::checked_from_start_len(start, len).unwrap());
            let owned = sub.to_bit_string();
            assert_eq!(owned.bit_len(), len);
            assert!(view_has_same_invariants(&owned));
            assert_eq!(owned.count_ones(), len);
        }
    }
}

#[test]
fn attack_bitstr_source_after_mutation() {
    // BitStr borrows source; verify it's correct even after the source is
    // consumed (but not mutated, since BitStr borrows it).
    let bits = bs("11001100");
    let view = bits.as_bit_str();

    // Create sub-views
    let sub = view.slice(UsizeCO::checked_from_start_len(2, 4).unwrap());
    assert_eq!(sub.to_bit_string().to_string(), "0011");

    // All views should be consistent
    assert_eq!(view.bit_len(), 8);
}

#[test]
fn attack_bitstr_slice_chain() {
    let bits = bs("1100110011");
    let v = bits.as_bit_str();
    let v1 = v.slice_from(2); // "00110011"
    let v2 = v1.slice_until(6); // "001100"
    let v3 = v2.slice(UsizeCO::checked_from_start_len(2, 2).unwrap()); // "11"

    assert_eq!(v3.to_bit_string().to_string(), "11");
    assert_eq!(v3.start(), 4); // 0 + 2 + 2 = 4
    assert_eq!(v3.bit_len(), 2);
}

// ===========================================================================
// 15. Extend attack vectors
// ===========================================================================

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

// ===========================================================================
// 16. Predicate attack vectors
// ===========================================================================

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
// 17. Stress: long chains of heterogenous operations
// ===========================================================================

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

// ===========================================================================
// 18. Memory / allocation edge cases
// ===========================================================================

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

// ===========================================================================
// 19. set_chunk precise attack
// ===========================================================================

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

// ===========================================================================
// 20. Prop-test style invariant checks
// ===========================================================================

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

// ===========================================================================
// 21. Diagnosed bugs — minimal reproducers
// ===========================================================================

/// Bug 1: `set_chunk` doesn't mask unused high bits after writing.
/// When `bit_start + len > self.bit_len`, bits beyond `bit_len` in the
/// last word get OR'd in but never masked, violating the invariant that
/// unused high bits are always zero.
#[test]
#[ignore = "BUG: set_chunk invariant break — reproducer (marker: B1)"]
fn diagnostic_set_chunk_invariant_break() {
    let mut bits = BitString::zeros(3);
    // Write 64 bits starting at position 0 — only bits 0-2 are valid.
    bits.set_chunk(0, u64::MAX, 64);
    // Unused bits in the last word should be zero, but they aren't.
    assert!(!view_has_same_invariants(&bits));
}

/// Bug 2: `BitStr::count_ones` doesn't mask the last partial word.
/// When a BitStr view has length not divisible by 64, the count includes
/// bits from the source beyond the view's end.
#[test]
#[ignore = "BUG: BitStr::count_ones partial word mask — reproducer (marker: B2)"]
fn diagnostic_bitstr_count_ones_last_word_mask() {
    // A 20-bit pattern with 10 ones
    let bits: BitString = "01010101010101010101".parse().unwrap();
    let view = bits.as_bit_str();

    // Slice just the first bit (which is '0')
    let sub = view.slice(UsizeCO::checked_from_start_len(0, 1).unwrap());
    assert_eq!(sub.bit_len(), 1);
    // Should be 0, but returns 10 (all the ones in the source's first word)
    let ones = sub.count_ones();
    assert_eq!(ones, 0, "BUG: count_ones({sub}) = {ones}, expected 0");
}

/// Bug 3: `BitStr::find` on unaligned starts misses occurrences that
/// straddle the unaligned→aligned word boundary.
#[test]
#[ignore = "BUG: BitStr::find blind spot at unaligned→aligned boundary — reproducer (marker: B3)"]
fn diagnostic_find_misses_cross_boundary_needle() {
    // "101" at positions 0, 63, and 126, with 60 zeros between each
    let bits: BitString = ("101".to_owned() + &"0".repeat(60) + "101" + &"0".repeat(60) + "101")
        .parse()
        .unwrap();

    let needle: BitString = "101".parse().unwrap();
    let needle_view = needle.as_bit_str();

    // First occurrence at 0 — found correctly
    assert_eq!(bits.find(needle_view), Some(0));

    // Second occurrence at 63 — let's check
    let remaining = bits.as_bit_str().slice_from(3);
    assert_eq!(
        remaining.find(needle_view),
        Some(60),
        "BUG: find on slice_from(3) should return Some(60), got {:?}",
        remaining.find(needle_view)
    );
}
