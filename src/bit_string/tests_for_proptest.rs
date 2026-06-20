//! Property-based adversarial tests: random operation sequences bounded
//! to avoid explosion, attempting to break BitString invariants.

use proptest::collection::vec;
use proptest::prelude::*;

use int_interval::UsizeCO;

use crate::BitString;

// ---------------------------------------------------------------------------
// Configuration — limits to prevent combinatorial explosion
// ---------------------------------------------------------------------------

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 1024,
        max_shrink_iters: 256,
        ..ProptestConfig::default()
    }
}

const MAX_BITS: usize = 256;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn any_bit_string() -> impl Strategy<Value = BitString> {
    (0..=MAX_BITS).prop_flat_map(|n| vec(any::<bool>(), n).prop_map(BitString::from_bool_iter))
}

fn same_len_pair() -> impl Strategy<Value = (BitString, BitString)> {
    (0..=MAX_BITS).prop_flat_map(|n| {
        (vec(any::<bool>(), n), vec(any::<bool>(), n))
            .prop_map(|(a, b)| (BitString::from_bool_iter(a), BitString::from_bool_iter(b)))
    })
}

/// Strategy for (bit_string, random_index_within).
fn bs_with_index() -> impl Strategy<Value = (BitString, usize)> {
    any_bit_string().prop_flat_map(|bs| {
        let bit_len = bs.bit_len();
        let idx_strat: BoxedStrategy<usize> = if bit_len > 0 {
            (0..bit_len).boxed()
        } else {
            Just(0).boxed()
        };
        (Just(bs), idx_strat)
    })
}

// ---------------------------------------------------------------------------
// Core invariants
// ---------------------------------------------------------------------------

fn assert_all_invariants(bits: &BitString) {
    let bit_len = bits.bit_len();
    let expected_words = crate::word_len(bit_len);
    let actual_words = bits.as_words().len();
    assert_eq!(
        actual_words, expected_words,
        "word-count invariant: bit_len={bit_len}, words={actual_words}, expected={expected_words}",
    );

    // Unused high bits in last word must be zero.
    let rem = bit_len % crate::WORD_BITS;
    if rem != 0 {
        if let Some(&last) = bits.as_words().last() {
            assert_eq!(
                last >> rem,
                0,
                "unused-bits invariant: last word has high bits set: {last:#b}, rem={rem}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn new_is_empty(_: ()) {
        let bits = BitString::new();
        assert_all_invariants(&bits);
        assert_eq!(bits.bit_len(), 0);
    }

    #[test]
    fn repeat_is_valid(value: bool, bit_len in 0usize..=MAX_BITS) {
        let bits = BitString::repeat(value, bit_len);
        assert_all_invariants(&bits);
        assert_eq!(bits.bit_len(), bit_len);
        if bit_len > 0 {
            assert_eq!(bits.count_ones(), if value { bit_len } else { 0 });
        }
    }

    #[test]
    fn ones_zeros_are_valid(bit_len in 0usize..=MAX_BITS) {
        let o = BitString::ones(bit_len);
        assert_all_invariants(&o);
        assert_eq!(o.count_ones(), bit_len);

        let z = BitString::zeros(bit_len);
        assert_all_invariants(&z);
        assert_eq!(z.count_ones(), 0);
    }

    #[test]
    fn from_bool_iter_is_valid(bools in vec(any::<bool>(), 0..MAX_BITS)) {
        let bits = BitString::from_bool_iter(bools);
        assert_all_invariants(&bits);
    }

    #[test]
    fn clone_preserves_invariant(bits in any_bit_string()) {
        let c = bits.clone();
        assert_all_invariants(&c);
        assert_eq!(bits, c);
    }
}

// ---------------------------------------------------------------------------
// Set / get round-trip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn set_then_get_is_consistent((bits, idx) in bs_with_index(), value: bool) {
        if bits.is_empty() { return Ok(()); }
        let mut bits = bits;
        bits.set(idx, value);
        assert_eq!(bits.get(idx), Some(value));
        assert_all_invariants(&bits);
    }

    #[test]
    fn set_chunk_get_chunk_roundtrip(bits in any_bit_string()) {
        let bit_len = bits.bit_len();
        if bit_len == 0 { return Ok(()); }
        // Pick a random start+len pair within bounds.
        let starts = [0, 1, bit_len / 2, bit_len.saturating_sub(1), bit_len.saturating_sub(64).min(bit_len)];
        for &start in &starts {
            let start = start.min(bit_len.saturating_sub(1));
            let max_len = (bit_len - start).min(64);
            if max_len == 0 { continue; }
            let len = max_len;
            let mut copy = bits.clone();
            let value = copy.get_chunk(start);
            copy.set_chunk(start, value, len);
            let roundtrip = copy.get_chunk(start);
            assert_eq!(value & crate::low_mask(len), roundtrip & crate::low_mask(len),
                "set_chunk mismatch at start={start} len={len}");
            assert_all_invariants(&copy);
        }
    }
}

// ---------------------------------------------------------------------------
// Push / pop
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn push_then_pop_is_identity(mut bits in any_bit_string(), value: bool) {
        let orig_len = bits.bit_len();
        prop_assume!(orig_len < MAX_BITS);

        bits.push(value);
        assert_all_invariants(&bits);
        assert_eq!(bits.bit_len(), orig_len + 1);
        let popped = bits.pop();
        assert_all_invariants(&bits);
        assert_eq!(popped, Some(value));
        assert_eq!(bits.bit_len(), orig_len);
    }

    #[test]
    fn pop_then_push_is_identity(mut bits in any_bit_string()) {
        let orig_len = bits.bit_len();
        prop_assume!(orig_len > 0 && orig_len < MAX_BITS);

        let last = bits.pop().unwrap();
        bits.push(last);
        assert_all_invariants(&bits);
        assert_eq!(bits.bit_len(), orig_len);
    }
}

// ---------------------------------------------------------------------------
// Insert / remove
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn insert_increments_len(mut bits in any_bit_string(), value: bool, index in 0usize..=MAX_BITS) {
        let orig_len = bits.bit_len();
        prop_assume!(orig_len < MAX_BITS);
        let idx = index.min(orig_len);
        bits.insert(idx, value);
        assert_all_invariants(&bits);
        assert_eq!(bits.bit_len(), orig_len + 1);
        assert_eq!(bits.get(idx), Some(value));
    }

    #[test]
    fn remove_decrements_len((bits, idx) in bs_with_index()) {
        if bits.is_empty() { return Ok(()); }
        let orig_len = bits.bit_len();
        let mut bits = bits;
        bits.remove(idx);
        assert_all_invariants(&bits);
        assert_eq!(bits.bit_len(), orig_len - 1);
    }
}

// ---------------------------------------------------------------------------
// Truncate
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn truncate_preserves_prefix(bits in any_bit_string(), new_len: usize) {
        let orig_len = bits.bit_len();
        let target = new_len.min(orig_len);

        let prefix: alloc::vec::Vec<bool> = (0..target)
            .map(|i| bits.get(i).unwrap()).collect();

        let mut copy = bits;
        copy.truncate(new_len);
        assert_all_invariants(&copy);
        assert_eq!(copy.bit_len(), target);

        for (i, expected) in prefix.iter().enumerate() {
            assert_eq!(copy.get(i), Some(*expected), "truncate lost prefix bit at i={i}");
        }
    }
}

// ---------------------------------------------------------------------------
// Binary ops
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn and_preserves_invariant((a, b) in same_len_pair()) {
        let r = a.and(&b).unwrap();
        assert_all_invariants(&r);
        assert_eq!(r.bit_len(), a.bit_len());
    }

    #[test]
    fn or_preserves_invariant((a, b) in same_len_pair()) {
        let r = a.or(&b).unwrap();
        assert_all_invariants(&r);
        assert_eq!(r.bit_len(), a.bit_len());
    }

    #[test]
    fn xor_preserves_invariant((a, b) in same_len_pair()) {
        let r = a.xor(&b).unwrap();
        assert_all_invariants(&r);
        assert_eq!(r.bit_len(), a.bit_len());
    }

    #[test]
    fn and_assign_preserves_invariant((mut a, b) in same_len_pair()) {
        a.and_assign(&b).unwrap();
        assert_all_invariants(&a);
    }

    #[test]
    fn or_assign_preserves_invariant((mut a, b) in same_len_pair()) {
        a.or_assign(&b).unwrap();
        assert_all_invariants(&a);
    }

    #[test]
    fn xor_assign_preserves_invariant((mut a, b) in same_len_pair()) {
        a.xor_assign(&b).unwrap();
        assert_all_invariants(&a);
    }
}

// ---------------------------------------------------------------------------
// Unary / shift ops
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn not_preserves_invariant(bits in any_bit_string()) {
        let bit_len = bits.bit_len();
        let r = bits.not();
        assert_all_invariants(&r);
        assert_eq!(r.bit_len(), bit_len);
    }

    #[test]
    fn not_is_involution(bits in any_bit_string()) {
        let n = bits.not();
        let nn = n.not();
        assert_eq!(bits, nn, "not(not(x)) must equal x");
    }

    #[test]
    fn count_ones_bounded(bits in any_bit_string()) {
        assert!(bits.count_ones() <= bits.bit_len());
    }

    #[test]
    fn count_ones_plus_zeros(bits in any_bit_string()) {
        assert_eq!(bits.count_ones() + bits.count_zeros(), bits.bit_len());
    }

    #[test]
    fn shl_preserves_invariant(bits in any_bit_string(), amount in 0usize..=256) {
        let bit_len = bits.bit_len();
        let r = bits.shl(amount);
        assert_all_invariants(&r);
        assert_eq!(r.bit_len(), bit_len);
    }

    #[test]
    fn shr_preserves_invariant(bits in any_bit_string(), amount in 0usize..=256) {
        let bit_len = bits.bit_len();
        let r = bits.shr(amount);
        assert_all_invariants(&r);
        assert_eq!(r.bit_len(), bit_len);
    }

    #[test]
    fn shl_assign_preserves_invariant(mut bits in any_bit_string(), amount in 0usize..=256) {
        let bit_len = bits.bit_len();
        bits.shl_assign(amount);
        assert_all_invariants(&bits);
        assert_eq!(bits.bit_len(), bit_len);
    }

    #[test]
    fn shr_assign_preserves_invariant(mut bits in any_bit_string(), amount in 0usize..=256) {
        let bit_len = bits.bit_len();
        bits.shr_assign(amount);
        assert_all_invariants(&bits);
        assert_eq!(bits.bit_len(), bit_len);
    }
}

// ---------------------------------------------------------------------------
// Editing ops
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn slice_preserves_invariant(bits in any_bit_string(), start in 0usize..=MAX_BITS, len in 0usize..=128) {
        let end = bits.bit_len().min(start.saturating_add(len));
        if start >= end { return Ok(()); }
        let interval = UsizeCO::try_new(start, end).unwrap();
        let r = bits.slice(interval);
        assert_all_invariants(&r);
    }

    #[test]
    fn extend_preserves_invariant(
        (n1, n2) in (0usize..64).prop_flat_map(|n1|
            (Just(n1), 0usize..(128usize.saturating_sub(n1)).min(64))
        ),
    ) {
        let bits = BitString::ones(n1);
        let ext = BitString::zeros(n2);
        let mut copy = bits.clone();
        copy.extend(&ext);
        assert_all_invariants(&copy);
        assert_eq!(copy.bit_len(), n1 + n2);
    }

    #[test]
    fn replace_preserves_invariant(
        bits in any_bit_string(),
        replacement in any_bit_string(),
    ) {
        let bit_len = bits.bit_len();
        if bit_len == 0 { return Ok(()); }
        // Pick a bounded interval.
        let start = (bit_len as u64 % 17) as usize % bit_len;
        let len = 8.min(bit_len - start);
        if len == 0 { return Ok(()); }
        let end = start + len;
        let interval = UsizeCO::try_new(start, end).unwrap();
        let r = bits.replace_interval(interval, &replacement);
        assert_all_invariants(&r);
    }

    #[test]
    fn retain_preserves_invariant(mut bits in any_bit_string()) {
        let bit_len = bits.bit_len();
        if bit_len == 0 { return Ok(()); }

        // Retain only set bits.
        bits.retain(|b| b);
        assert_all_invariants(&bits);
    }
}

// ---------------------------------------------------------------------------
// Random operation sequences
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Op {
    Push(bool),
    Pop,
    Set { index: usize, value: bool },
    Insert { index: usize, value: bool },
    Truncate(usize),
    NotInPlace,
}

fn op_strat() -> impl Strategy<Value = Op> {
    prop_oneof![
        4 => any::<bool>().prop_map(Op::Push),
        2 => Just(Op::Pop),
        4 => (0usize..MAX_BITS, any::<bool>())
            .prop_map(|(i, v)| Op::Set { index: i, value: v }),
        2 => (0usize..=MAX_BITS, any::<bool>())
            .prop_map(|(i, v)| Op::Insert { index: i, value: v }),
        2 => (0usize..MAX_BITS).prop_map(Op::Truncate),
        2 => Just(Op::NotInPlace),
    ]
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn random_op_sequences_preserve_invariants(
        mut bits in any_bit_string(),
        ops in vec(op_strat(), 1..32),
    ) {
        for op in &ops {
            let bit_len = bits.bit_len();
            match op {
                Op::Push(v) => if bit_len < MAX_BITS { bits.push(*v) },
                Op::Pop => drop((bit_len > 0).then(|| bits.pop())),
                Op::Set { index, value } => { bits.set((*index).min(bit_len), *value); },
                Op::Insert { index, value } => {
                    if bit_len < MAX_BITS { bits.insert((*index).min(bit_len), *value); }
                },
                Op::Truncate(n) => bits.truncate(*n),
                Op::NotInPlace => bits.not_assign(),
            }
            assert_all_invariants(&bits);
        }
    }
}
