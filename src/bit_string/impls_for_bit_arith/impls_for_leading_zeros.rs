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
        let words = self.words();

        // ── First-word fast path ──────────────────────────────────
        // Catches dense / alternating / any early-1 input directly,
        // avoiding the trait call overhead entirely.
        let w0 = words[0];
        if w0 != 0 {
            return (w0.trailing_zeros() as usize).min(bit_len);
        }

        // ── Tiny inputs — inline scalar ───────────────────────────
        // For < SMALL_WORDS full words, the trait dispatch overhead
        // dominates.  Keep the hot tiny path here.
        let last_wi = (bit_len - 1) / WORD_BITS;
        let end_rem = bit_len % WORD_BITS;
        let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
        if mid_end < SMALL_WORDS {
            let mut scanned = WORD_BITS; // word 0 already checked above
            for i in 1..mid_end {
                let w = words[i];
                if w != 0 {
                    return (scanned + w.trailing_zeros() as usize).min(bit_len);
                }
                scanned += WORD_BITS;
            }
            if end_rem != 0 {
                let last = words[mid_end] & ((1u64 << end_rem).wrapping_sub(1));
                if last == 0 {
                    return bit_len;
                }
                return (scanned + last.trailing_zeros() as usize).min(bit_len);
            }
            return bit_len;
        }

        // ── SIMD via trait ────────────────────────────────────────
        // All full SIMD logic lives in `WordsScan::leading_value_bits`.
        words.leading_value_bits::<FILL_ZEROS, true>(0, bit_len)
    }

    /// Returns the number of consecutive `true` bits from the start.
    #[inline]
    pub fn leading_ones(&self) -> usize {
        let bit_len = self.bit_len;
        if bit_len == 0 {
            return 0;
        }
        let words = self.words();

        let w0 = words[0];
        if w0 != u64::MAX {
            return ((!w0).trailing_zeros() as usize).min(bit_len);
        }

        let last_wi = (bit_len - 1) / WORD_BITS;
        let end_rem = bit_len % WORD_BITS;
        let mid_end = if end_rem == 0 { last_wi + 1 } else { last_wi };
        if mid_end < SMALL_WORDS {
            let mut scanned = WORD_BITS;
            for i in 1..mid_end {
                let w = words[i];
                if w != u64::MAX {
                    return (scanned + (!w).trailing_zeros() as usize).min(bit_len);
                }
                scanned += WORD_BITS;
            }
            if end_rem != 0 {
                let last = words[mid_end] & ((1u64 << end_rem).wrapping_sub(1));
                if last == ((1u64 << end_rem).wrapping_sub(1)) {
                    return bit_len;
                }
                return (scanned + (!last).trailing_zeros() as usize).min(bit_len);
            }
            return bit_len;
        }

        words.leading_value_bits::<FILL_ONES, true>(0, bit_len)
    }

    /// Returns the number of consecutive `false` bits from the end.
    #[inline]
    pub fn trailing_zeros(&self) -> usize {
        self.words()
            .trailing_value_bits::<FILL_ZEROS, true>(0, self.bit_len)
    }

    /// Returns the number of consecutive `true` bits from the end.
    #[inline]
    pub fn trailing_ones(&self) -> usize {
        self.words()
            .trailing_value_bits::<FILL_ONES, true>(0, self.bit_len)
    }
}
