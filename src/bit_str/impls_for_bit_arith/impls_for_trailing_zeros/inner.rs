//! `WORD_ALIGNED = true` is a caller guarantee; `false` makes no alignment
//! guarantee and retains the general path.

use crate::BitStr;
use crate::WORD_BITS;
use crate::traits::WordsScan;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub(crate) fn trailing_value_bits_inner<const FILL: u64, const WORD_ALIGNED: bool>(
        &self,
    ) -> usize {
        debug_assert!(!WORD_ALIGNED || self.start.is_multiple_of(WORD_BITS));
        if self.bit_len == 0 {
            return 0;
        }
        let all_words = self.source.words();
        let word_start = self.start / WORD_BITS;
        // SAFETY: BitStr invariants guarantee `start` and `bit_len` are
        // within the source BitString's bounds.
        let words_ptr = unsafe { all_words.as_ptr().add(word_start) };
        let start_offset = (self.start % WORD_BITS) as u32;

        // Aligned one- and two-word views avoid rebuilding a slice and
        // entering the generic reverse scanner.
        if WORD_ALIGNED && self.bit_len <= WORD_BITS * 2 {
            let last_wi = (self.bit_len - 1) / WORD_BITS;
            let used = self.bit_len - last_wi * WORD_BITS;
            let mask = if used == WORD_BITS {
                u64::MAX
            } else {
                (1u64 << used) - 1
            };
            // SAFETY: BitStr invariants guarantee storage for last_wi.
            let last = unsafe { *words_ptr.add(last_wi) };
            let mismatch = (last ^ FILL) & mask;
            if mismatch != 0 {
                return (mismatch << (WORD_BITS - used)).leading_zeros() as usize;
            }
            if last_wi == 0 {
                return self.bit_len;
            }
            // SAFETY: last_wi == 1, so the first word exists as well.
            let first_mismatch = unsafe { *words_ptr } ^ FILL;
            if first_mismatch == 0 {
                return self.bit_len;
            }
            return used + first_mismatch.leading_zeros() as usize;
        }

        // Fast-path: check the rightmost word(s) before the trait call.
        // Mirrors the shortcuts in BitString's trailing section.

        let end_offset = start_offset as usize + self.bit_len;
        let end_rem = end_offset % WORD_BITS;
        let last_wi = (end_offset - 1) / WORD_BITS;

        // ── Last partial word ────────────────────────────────────
        if end_rem != 0 {
            let last_limit = if last_wi == 0 {
                end_rem - start_offset as usize
            } else {
                end_rem
            };
            // SAFETY: `last_wi` is within bounds per the BitStr invariant.
            let raw_last = unsafe { *words_ptr.add(last_wi) };
            let last_val = if last_wi == 0 {
                raw_last >> start_offset
            } else {
                raw_last & ((1u64 << end_rem).wrapping_sub(1))
            };
            let shifted = if FILL == 0 {
                last_val << (WORD_BITS - last_limit)
            } else {
                (!last_val) << (WORD_BITS - last_limit)
            };
            let last_count = (shifted.leading_zeros() as usize).min(last_limit);
            if last_count < last_limit {
                return last_count;
            }
            if last_wi == 0 {
                return self.bit_len;
            }
        }

        // ── Rightmost full word ──────────────────────────────────
        if self.bit_len > WORD_BITS {
            let last_full = if end_rem != 0 { last_wi - 1 } else { last_wi };
            // SAFETY: `last_full` is within bounds.
            let w = unsafe { *words_ptr.add(last_full) };
            if w != FILL {
                let tail = if end_rem != 0 { end_rem } else { 0 };
                let count = if FILL == 0 {
                    w.leading_zeros() as usize
                } else {
                    (!w).leading_zeros() as usize
                };
                return (tail + count).min(self.bit_len);
            }
        }

        // SAFETY: `words_ptr` is derived from `all_words.as_ptr()` with
        // `word_start ≤ all_words.len()` (guaranteed by BitStr invariant).
        // The sub-slice `all_words.len() - word_start` stays within the
        // allocation and is only used for the duration of the trait
        // method call below.
        let words = unsafe { core::slice::from_raw_parts(words_ptr, all_words.len() - word_start) };
        words.trailing_value_bits::<FILL, WORD_ALIGNED>(start_offset, self.bit_len)
    }
}
