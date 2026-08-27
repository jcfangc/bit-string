use crate::traits::WordsScan;
use crate::{FILL_ONES, FILL_ZEROS, WORD_BITS};

use super::BitString;

impl BitString {
    /// Returns the number of consecutive `false` bits from the end.
    #[inline]
    pub fn trailing_zeros(&self) -> usize {
        let bit_len = self.bit_len;
        if bit_len == 0 {
            return 0;
        }
        let words_ptr = self.words.as_ptr();

        if let Some(count) = unsafe { trailing_small::<FILL_ZEROS>(words_ptr, bit_len) } {
            return count;
        }

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

        if let Some(count) = unsafe { trailing_small::<FILL_ONES>(words_ptr, bit_len) } {
            return count;
        }

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

/// Handles owned strings that fit in at most two words without entering the
/// generic reverse scanner.
///
/// # Safety
///
/// `words_ptr` must point to the backing storage for `bit_len` bits.
#[inline]
unsafe fn trailing_small<const FILL: u64>(words_ptr: *const u64, bit_len: usize) -> Option<usize> {
    if bit_len > WORD_BITS * 2 {
        return None;
    }

    let last_wi = (bit_len - 1) / WORD_BITS;
    let used = bit_len - last_wi * WORD_BITS;
    let mask = if used == WORD_BITS {
        u64::MAX
    } else {
        (1u64 << used) - 1
    };
    // SAFETY: the caller guarantees backing storage for every covered word.
    let last = unsafe { *words_ptr.add(last_wi) };
    let mismatch = (last ^ FILL) & mask;
    if mismatch != 0 {
        return Some((mismatch << (WORD_BITS - used)).leading_zeros() as usize);
    }
    if last_wi == 0 {
        return Some(bit_len);
    }

    // SAFETY: last_wi == 1, so the first backing word also exists.
    let first_mismatch = unsafe { *words_ptr } ^ FILL;
    if first_mismatch == 0 {
        Some(bit_len)
    } else {
        Some(used + first_mismatch.leading_zeros() as usize)
    }
}
