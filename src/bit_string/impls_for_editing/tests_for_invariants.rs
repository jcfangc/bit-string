//! Adversarial tests: try to break BitString invariants through editing
//! operations at extreme values, word boundaries, and edge cases.

use alloc::string::ToString;

use crate::BitString;
use crate::word_len;
use int_interval::UsizeCO;

/// Verify that `self.as_words().len() == word_len(self.bit_len())`.
fn assert_invariants(bits: &BitString) {
    let bit_len = bits.bit_len();
    let expected_words = word_len(bit_len);
    let actual_words = bits.words().len();
    assert_eq!(
        actual_words, expected_words,
        "word count mismatch: bit_len={bit_len}, words={actual_words}, expected_words={expected_words}",
    );
}

// ---------------------------------------------------------------------------
// Empty / zero-length edge cases
// ---------------------------------------------------------------------------

#[test]
fn truncate_down_to_empty() {
    let mut bits = BitString::ones(4);
    bits.truncate(0);
    assert_invariants(&bits);
    assert!(bits.is_empty());
}

#[test]
fn truncate_to_same_length_is_noop() {
    let mut bits = BitString::ones(8);
    let bit_len = bits.bit_len();
    bits.truncate(bit_len);
    assert_invariants(&bits);
    assert_eq!(bits.to_string(), "11111111");
}

#[test]
fn truncate_to_larger_length_is_noop() {
    let mut bits = BitString::ones(3);
    bits.truncate(usize::MAX);
    assert_invariants(&bits);
    assert_eq!(bits.to_string(), "111");
}

#[test]
fn clear_on_already_empty_is_idempotent() {
    let mut bits = BitString::new();
    bits.clear();
    bits.clear();
    assert!(bits.is_empty());
    assert_eq!(bits.bit_len(), 0);
    assert_eq!(bits.words().len(), 0);
}

// ---------------------------------------------------------------------------
// Word-boundary truncate
// ---------------------------------------------------------------------------

#[test]
fn truncate_across_word_boundary_preserves_invariant() {
    for bit_len in [63, 64, 65, 127, 128, 129] {
        let orig = BitString::ones(bit_len);
        for target in [0, 1, 63, 64, 65, bit_len - 1] {
            if target > bit_len {
                continue;
            }
            let mut bits = orig.clone();
            bits.truncate(target);
            assert_invariants(&bits);
            assert_eq!(bits.bit_len(), target, "truncate {bit_len} → {target}");
        }
    }
}

// ---------------------------------------------------------------------------
// Push/pop at word boundaries
// ---------------------------------------------------------------------------

#[test]
fn push_pop_at_word_boundary_preserves_invariant() {
    let mut bits = BitString::ones(64);
    bits.push(true);
    assert_invariants(&bits);
    assert_eq!(bits.bit_len(), 65);
    assert_eq!(bits.pop(), Some(true));
    assert_invariants(&bits);
    assert_eq!(bits.bit_len(), 64);

    for _ in 0..64 {
        bits.pop();
    }
    assert_invariants(&bits);
    assert!(bits.is_empty());
    assert_eq!(bits.pop(), None);
}

#[test]
fn push_false_and_pop_on_empty() {
    let mut bits = BitString::new();
    bits.push(false);
    assert_eq!(bits.bit_len(), 1);
    assert_invariants(&bits);
    bits.push(true);
    assert_eq!(bits.bit_len(), 2);
    assert_invariants(&bits);
    assert_eq!(bits.pop(), Some(true));
    assert_eq!(bits.pop(), Some(false));
    assert_eq!(bits.pop(), None);
    assert!(bits.is_empty());
}

// ---------------------------------------------------------------------------
// Insert/remove at word boundaries
// ---------------------------------------------------------------------------

#[test]
fn insert_bit_at_word_boundary() {
    for start_len in [63, 64, 65, 127, 128, 129] {
        for index in [0, start_len] {
            let mut bits = BitString::ones(start_len);
            bits.insert(index, false);
            assert_invariants(&bits);
            assert_eq!(bits.bit_len(), start_len + 1);
            assert!(!bits.get(index).unwrap());
        }
    }
}

#[test]
fn insert_bit_string_at_word_boundary() {
    let filler = BitString::zeros(32);
    for host_len in [63, 64, 65] {
        for index in [0, host_len] {
            let mut bits = BitString::ones(host_len);
            bits.insert_bit_string(index, &filler);
            assert_invariants(&bits);
            assert_eq!(bits.bit_len(), host_len + 32);
        }
    }
}

#[test]
fn remove_at_word_boundary() {
    for start_len in [64, 65, 128, 129] {
        let mut bits = BitString::ones(start_len);
        bits.remove(0);
        assert_invariants(&bits);
        assert_eq!(bits.bit_len(), start_len - 1);
        assert!(bits.get(0).unwrap());
    }
}

// ---------------------------------------------------------------------------
// Replace interval extremes
// ---------------------------------------------------------------------------

#[test]
fn replace_with_empty_replacement() {
    let bits = BitString::ones(8);
    let empty = BitString::new();
    let r = bits.replace_interval(UsizeCO::try_new(2, 6).unwrap(), &empty);
    assert_invariants(&r);
    assert_eq!(r.to_string(), "1111");
}

#[test]
fn replace_full_range() {
    let bits = BitString::ones(4);
    let repl = BitString::zeros(8);
    let r = bits.replace_interval(UsizeCO::try_new(0, 4).unwrap(), &repl);
    assert_invariants(&r);
    assert_eq!(r.to_string(), "00000000");
}

#[test]
fn replace_out_of_bounds_interval_is_clamped() {
    let bits = BitString::ones(4);
    let repl = BitString::ones(3);
    let r = bits.replace_interval(UsizeCO::try_new(10, 20).unwrap(), &repl);
    assert_invariants(&r);
    // Clamped: interval (10,20) → (4,4). An interval that clamps to an
    // empty span at bit_len behaves like an append.
    assert_eq!(r.to_string(), "1111111");
}

// ---------------------------------------------------------------------------
// Drain interval extremes
// ---------------------------------------------------------------------------

#[test]
fn drain_entire_contents() {
    let bits = BitString::ones(8);
    let d = bits.drain_interval(UsizeCO::try_new(0, 8).unwrap());
    assert_invariants(&d);
    assert!(d.is_empty());
}

#[test]
fn drain_past_end_is_clamped() {
    let bits = BitString::ones(4);
    let d = bits.drain_interval(UsizeCO::try_new(2, 100).unwrap());
    assert_invariants(&d);
    assert_eq!(d.to_string(), "11");
}

#[test]
fn drain_empty_interval_returns_clone() {
    let bits = BitString::ones(4);
    // Interval directly past the end clamps to empty → unchanged
    let d = bits.drain_interval(UsizeCO::try_new(4, 5).unwrap());
    assert_invariants(&d);
    assert_eq!(d.to_string(), "1111");
}

// ---------------------------------------------------------------------------
// Slice edge cases
// ---------------------------------------------------------------------------

#[test]
fn slice_at_word_boundaries() {
    let bits = BitString::ones(64);
    for (start, end) in [(0, 64), (0, 63), (1, 64), (63, 64)] {
        let interval = UsizeCO::try_new(start, end).unwrap();
        let s = bits.slice(interval);
        assert_invariants(&s);
    }
}

#[test]
fn slice_beyond_len_is_clamped() {
    let bits = BitString::ones(4);
    let s = bits.slice(UsizeCO::try_new(0, 100).unwrap());
    assert_invariants(&s);
    assert_eq!(s.to_string(), "1111");
}

// ---------------------------------------------------------------------------
// Extend edge cases
// ---------------------------------------------------------------------------

#[test]
fn extend_by_empty_is_noop() {
    let mut bits = BitString::ones(3);
    bits.extend(&BitString::new());
    assert_invariants(&bits);
    assert_eq!(bits.to_string(), "111");
}

#[test]
fn empty_extended_by_nonempty_works() {
    let mut bits = BitString::new();
    bits.extend(&BitString::ones(3));
    assert_invariants(&bits);
    assert_eq!(bits.to_string(), "111");
}

#[test]
fn extend_across_word_boundary() {
    for host_len in [63, 64, 65] {
        for ext_len in [1, 63, 64, 65] {
            let mut bits = BitString::ones(host_len);
            let ext = BitString::zeros(ext_len);
            bits.extend(&ext);
            assert_invariants(&bits);
            assert_eq!(
                bits.bit_len(),
                host_len + ext_len,
                "host={host_len} + ext={ext_len}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Repeat edge cases
// ---------------------------------------------------------------------------

#[test]
fn repeat_zero_and_word_boundary_lengths() {
    let b = BitString::repeat(true, 0);
    assert!(b.is_empty());
    assert_invariants(&b);

    let b = BitString::repeat(false, 63);
    assert_eq!(b.bit_len(), 63);
    assert_invariants(&b);

    let b = BitString::repeat(true, 64);
    assert_eq!(b.bit_len(), 64);
    assert_invariants(&b);
}

// ---------------------------------------------------------------------------
// Set / set_chunk edge cases
// ---------------------------------------------------------------------------

#[test]
fn set_all_bits_back_and_forth() {
    let mut bits = BitString::zeros(128);
    for i in 0..128 {
        bits.set(i, true);
    }
    assert_eq!(bits.count_ones(), 128);
    assert_invariants(&bits);

    for i in 0..128 {
        bits.set(i, false);
    }
    assert_eq!(bits.count_ones(), 0);
    assert_invariants(&bits);
}

#[test]
fn set_chunk_at_word_boundary_crosses_correctly() {
    let mut bits = BitString::zeros(128);
    bits.set_chunk(60, u64::MAX, 64);
    assert_invariants(&bits);
    // Bits 60..(60+64) = 60..124 should be set
    assert_eq!(bits.count_ones(), 64);
}

#[test]
fn set_chunk_with_zero_len_is_noop() {
    let mut bits = BitString::ones(64);
    bits.set_chunk(0, 0, 0);
    assert_invariants(&bits);
    assert_eq!(bits.count_ones(), 64);
}

// ---------------------------------------------------------------------------
// Chained editing + arithmetic invariants
// ---------------------------------------------------------------------------

#[test]
fn edit_then_not_then_edit_preserves_invariant() {
    let mut bits = BitString::ones(65);
    bits.set(0, false);
    bits.push(true);
    let negated = bits.not();
    assert_invariants(&negated);
    let mut roundtrip = negated.clone();
    roundtrip.not_assign();
    assert_invariants(&roundtrip);
    // After double negation, same as original (but masking may differ)
    assert_eq!(negated.not(), roundtrip);
}

#[test]
fn shift_then_edit_preserves_invariant() {
    for amount in [0, 1, 31, 63, 64, 65] {
        let bits = BitString::ones(128);
        let shifted = bits.shl(amount);
        assert_invariants(&shifted);
        assert_eq!(shifted.bit_len(), 128);

        let mut shifted_back = shifted.clone();
        shifted_back.shr_assign(amount);
        assert_invariants(&shifted_back);
    }
}

#[test]
fn binary_op_then_truncate_preserves_invariant() {
    let a = BitString::ones(128);
    let b = BitString::zeros(128);
    let mut xored = a.xor(&b).unwrap();
    assert_eq!(xored.count_ones(), 128);
    xored.truncate(65);
    assert_invariants(&xored);
    assert_eq!(xored.count_ones(), 65);
}
