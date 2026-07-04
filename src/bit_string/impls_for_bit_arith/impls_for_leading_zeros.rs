use crate::traits::WordsScan;
use crate::{FILL_ONES, FILL_ZEROS, SMALL_WORDS, WORD_BITS};

use super::BitString;

impl BitString {
    /// Returns the number of consecutive `false` bits from the start.
    #[inline]
    pub fn leading_zeros(&self) -> usize {
        let bit_len = self.bit_len;
        if bit_len == 0 {
            return 0;
        }
        // SAFETY: `words` is always non-empty when bit_len > 0
        // (BitString invariants guarantee at least one word).
        let words_ptr = self.words.as_ptr();

        // ── First-word fast path ──────────────────────────────────
        // SAFETY: `bit_len > 0` (guarded above), so `words` has at least
        // one element per the BitString invariant.
        let w0 = unsafe { *words_ptr };
        if w0 != 0 {
            return (w0.trailing_zeros() as usize).min(bit_len);
        }

        // ── Tiny inputs — dispatch to BMI1 when available ────────
        let last_wi = (bit_len - 1) / WORD_BITS;
        let end_rem = bit_len % WORD_BITS;
        let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
        if mid_end < SMALL_WORDS {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            if !cfg!(target_feature = "bmi1") && crate::cpuid::features().bmi1 {
                // SAFETY: BMI1 confirmed by CPUID.  `words_ptr` is valid
                // for at least `mid_end + (end_rem != 0) as usize` u64
                // reads per BitString invariants.
                return unsafe { leading_zeros_scalar_bmi(bit_len, words_ptr, mid_end, end_rem) };
            }
            let mut scanned = WORD_BITS; // word 0 already checked above
            // SAFETY: `i` ranges in `1..mid_end`.  `words` contains at
            // least `mid_end + (end_rem != 0) as usize` elements by the
            // BitString invariant (backing storage covers all bits).
            for i in 1..mid_end {
                let w = unsafe { *words_ptr.add(i) };
                if w != 0 {
                    return (scanned + w.trailing_zeros() as usize).min(bit_len);
                }
                scanned += WORD_BITS;
            }
            if end_rem != 0 {
                // SAFETY: `mid_end` is the index of the last partial word;
                // it is within bounds because `end_rem != 0` implies an
                // extra word exists beyond `mid_end - 1`.
                let last = unsafe { *words_ptr.add(mid_end) } & ((1u64 << end_rem).wrapping_sub(1));
                if last == 0 {
                    return bit_len;
                }
                return (scanned + last.trailing_zeros() as usize).min(bit_len);
            }
            return bit_len;
        }

        // ── SIMD via trait ────────────────────────────────────────
        self.words()
            .leading_value_bits::<FILL_ZEROS, true>(0, bit_len)
    }

    /// Returns the number of consecutive `true` bits from the start.
    #[inline]
    pub fn leading_ones(&self) -> usize {
        let bit_len = self.bit_len;
        if bit_len == 0 {
            return 0;
        }
        let words_ptr = self.words.as_ptr();

        // SAFETY: `bit_len > 0` (guarded above), so `words` has at least
        // one element per the BitString invariant.
        let w0 = unsafe { *words_ptr };
        if w0 != u64::MAX {
            return ((!w0).trailing_zeros() as usize).min(bit_len);
        }

        let last_wi = (bit_len - 1) / WORD_BITS;
        let end_rem = bit_len % WORD_BITS;
        let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
        if mid_end < SMALL_WORDS {
            let mut scanned = WORD_BITS;
            // SAFETY: `i < mid_end`.  `words` has at least
            // `mid_end + (end_rem != 0) as usize` elements.
            for i in 1..mid_end {
                let w = unsafe { *words_ptr.add(i) };
                if w != u64::MAX {
                    return (scanned + (!w).trailing_zeros() as usize).min(bit_len);
                }
                scanned += WORD_BITS;
            }
            if end_rem != 0 {
                // SAFETY: `mid_end` is the index of the last partial word
                // and is within bounds (backing storage covers all bits).
                let last = unsafe { *words_ptr.add(mid_end) } & ((1u64 << end_rem).wrapping_sub(1));
                if last == ((1u64 << end_rem).wrapping_sub(1)) {
                    return bit_len;
                }
                return (scanned + (!last).trailing_zeros() as usize).min(bit_len);
            }
            return bit_len;
        }

        self.words()
            .leading_value_bits::<FILL_ONES, true>(0, bit_len)
    }

    /// Returns the number of consecutive `false` bits from the end.
    #[inline]
    pub fn trailing_zeros(&self) -> usize {
        let bit_len = self.bit_len;
        if bit_len == 0 {
            return 0;
        }
        let words_ptr = self.words.as_ptr();

        // ── Last partial word ────────────────────────────────────
        let end_rem = bit_len % WORD_BITS;
        if end_rem != 0 {
            let last_wi = (bit_len - 1) / WORD_BITS;
            // SAFETY: `last_wi` is a valid index — `bit_len > 0` and
            // `words` always has ≥ (bit_len + 63) / 64 elements.
            let last = unsafe { *words_ptr.add(last_wi) } & ((1u64 << end_rem).wrapping_sub(1));
            if last != 0 {
                let shifted = last << (WORD_BITS - end_rem);
                return (shifted.leading_zeros() as usize).min(bit_len);
            }
        }

        // ── Rightmost full word ──────────────────────────────────
        if bit_len > WORD_BITS {
            let last_full = if end_rem != 0 {
                (bit_len - 1) / WORD_BITS - 1
            } else {
                (bit_len - 1) / WORD_BITS
            };
            // SAFETY: `last_full < (bit_len + 63) / 64`, so it is a valid
            // index into `words` (backing storage covers all bits).
            let w = unsafe { *words_ptr.add(last_full) };
            if w != 0 {
                let tail = if end_rem != 0 { end_rem } else { 0 };
                return (tail + w.leading_zeros() as usize).min(bit_len);
            }
        }

        self.words()
            .trailing_value_bits::<FILL_ZEROS, true>(0, bit_len)
    }

    /// Returns the number of consecutive `true` bits from the end.
    #[inline]
    pub fn trailing_ones(&self) -> usize {
        let bit_len = self.bit_len;
        if bit_len == 0 {
            return 0;
        }
        let words_ptr = self.words.as_ptr();

        let end_rem = bit_len % WORD_BITS;
        if end_rem != 0 {
            let last_wi = (bit_len - 1) / WORD_BITS;
            // SAFETY: `last_wi` is a valid index per the same invariant
            // as trailing_zeros (BitString backing storage covers all bits).
            let last = unsafe { *words_ptr.add(last_wi) } & ((1u64 << end_rem).wrapping_sub(1));
            if last != ((1u64 << end_rem).wrapping_sub(1)) {
                let shifted = (!last) << (WORD_BITS - end_rem);
                return (shifted.leading_zeros() as usize).min(bit_len);
            }
        }

        if bit_len > WORD_BITS {
            let last_full = if end_rem != 0 {
                (bit_len - 1) / WORD_BITS - 1
            } else {
                (bit_len - 1) / WORD_BITS
            };
            // SAFETY: `last_full` is a valid index into `words` per the
            // BitString backing-storage invariant.
            let w = unsafe { *words_ptr.add(last_full) };
            if w != u64::MAX {
                let tail = if end_rem != 0 { end_rem } else { 0 };
                return (tail + (!w).leading_zeros() as usize).min(bit_len);
            }
        }

        self.words()
            .trailing_value_bits::<FILL_ONES, true>(0, bit_len)
    }
}

// ── BMI1-accelerated scalar path ───────────────────────────────────────

/// BMI1 variant of the tiny-input scalar loop.  Uses `tzcnt` instead of
/// `bsf` for `trailing_zeros()`, eliminating the false output dependency.
///
/// # Safety
///
/// `words_ptr` must be valid for at least `mid_end + (end_rem != 0) as
/// usize` u64 reads.  Caller must guarantee BMI1 is available per CPUID.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "bmi1")]
unsafe fn leading_zeros_scalar_bmi(
    bit_len: usize,
    words_ptr: *const u64,
    mid_end: usize,
    end_rem: usize,
) -> usize {
    let mut scanned = crate::WORD_BITS; // word 0 already checked by caller
    // SAFETY: caller guarantees `words_ptr` is valid for the indices used.
    for i in 1..mid_end {
        let w = unsafe { *words_ptr.add(i) };
        if w != 0 {
            return (scanned + w.trailing_zeros() as usize).min(bit_len);
        }
        scanned += crate::WORD_BITS;
    }
    if end_rem != 0 {
        // SAFETY: `mid_end` is the index of the last partial word;
        // bounds guaranteed by caller.
        let last = unsafe { *words_ptr.add(mid_end) } & ((1u64 << end_rem).wrapping_sub(1));
        if last == 0 {
            return bit_len;
        }
        return (scanned + last.trailing_zeros() as usize).min(bit_len);
    }
    bit_len
}
