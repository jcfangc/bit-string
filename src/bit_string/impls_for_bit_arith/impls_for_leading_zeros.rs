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

        // One- and two-word strings are common enough to avoid the generic scan.
        if bit_len <= WORD_BITS {
            return bit_len;
        }
        if bit_len <= WORD_BITS * 2 {
            let used = bit_len - WORD_BITS;
            // SAFETY: bit_len > WORD_BITS, so the second backing word exists.
            let w1 = unsafe { *words_ptr.add(1) };
            let mask = if used == WORD_BITS {
                u64::MAX
            } else {
                (1u64 << used) - 1
            };
            let w1 = w1 & mask;
            if w1 == 0 {
                return bit_len;
            }
            return WORD_BITS + w1.trailing_zeros() as usize;
        }

        // ── Tiny inputs ───────────────────────────────────
        let last_wi = (bit_len - 1) / WORD_BITS;
        let end_rem = bit_len % WORD_BITS;
        let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
        if mid_end < SMALL_WORDS {
            let mut scanned = WORD_BITS; // word 0 already checked above
            // SAFETY: i ranges in 1..mid_end. words contains every
            // backing word covered by bit_len.
            for i in 1..mid_end {
                let w = unsafe { *words_ptr.add(i) };
                if w != 0 {
                    return (scanned + w.trailing_zeros() as usize).min(bit_len);
                }
                scanned += WORD_BITS;
            }
            if end_rem != 0 {
                // SAFETY: a partial final word exists at mid_end.
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
}
