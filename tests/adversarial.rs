//! Adversarial tests — attack every public API with edge cases, fuzz-style
//! inputs, and adversarial combinations that might trigger panics, incorrect
//! results, or invariant violations.

// rustc flags for NEON cross-check
#![cfg_attr(
    all(target_arch = "aarch64", target_feature = "neon"),
    feature(stdarch_aarch64_prefetch)
)]

use bit_string::BitString;
use core::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

pub(crate) fn hash<T: Hash>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

pub(crate) fn bs(s: &str) -> BitString {
    s.parse().unwrap()
}

/// Check internal invariants: bit_len matches actual bit count; unused bits
/// in the last word are zeroed.
pub(crate) fn view_has_same_invariants(bits: &BitString) -> bool {
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

// ---------------------------------------------------------------------------
// Module declarations
// ---------------------------------------------------------------------------

#[path = "adversarial/tests_for_access.rs"]
mod tests_for_access;
#[path = "adversarial/tests_for_bitstr.rs"]
mod tests_for_bitstr;
#[path = "adversarial/tests_for_bitwise.rs"]
mod tests_for_bitwise;
#[path = "adversarial/tests_for_bugs.rs"]
mod tests_for_bugs;
#[path = "adversarial/tests_for_concat.rs"]
mod tests_for_concat;
#[path = "adversarial/tests_for_construction.rs"]
mod tests_for_construction;
#[path = "adversarial/tests_for_count.rs"]
mod tests_for_count;
#[path = "adversarial/tests_for_editing.rs"]
mod tests_for_editing;
#[path = "adversarial/tests_for_fmt.rs"]
mod tests_for_fmt;
#[path = "adversarial/tests_for_iter.rs"]
mod tests_for_iter;
#[path = "adversarial/tests_for_matching.rs"]
mod tests_for_matching;
#[path = "adversarial/tests_for_ord_hash.rs"]
mod tests_for_ord_hash;
#[path = "adversarial/tests_for_predicates.rs"]
mod tests_for_predicates;
#[path = "adversarial/tests_for_replace.rs"]
mod tests_for_replace;
#[path = "adversarial/tests_for_retain.rs"]
mod tests_for_retain;
#[path = "adversarial/tests_for_shift.rs"]
mod tests_for_shift;
#[path = "adversarial/tests_for_slice_drain.rs"]
mod tests_for_slice_drain;
#[path = "adversarial/tests_for_stress.rs"]
mod tests_for_stress;
