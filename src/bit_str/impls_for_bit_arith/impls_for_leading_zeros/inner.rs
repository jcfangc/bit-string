use crate::BitStr;
use crate::WORD_BITS;
use crate::traits::WordsScan;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub(crate) fn leading_value_bits_inner<const FILL: u64, const WORD_ALIGNED: bool>(
        &self,
    ) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        let all_words = self.source.words();
        let word_start = self.start / WORD_BITS;
        // SAFETY: BitStr invariants guarantee `start` and `bit_len` are
        // within the source BitString's bounds; `word_start` is ≤ all_words.len().
        let words_ptr = unsafe { all_words.as_ptr().add(word_start) };
        let start_offset = (self.start % WORD_BITS) as u32;
        // SAFETY: `words_ptr` is valid for at least 1 u64 read
        // (bit_len > 0 ⇒ at least one word exists).
        let w0 = unsafe { *words_ptr };

        // First-word fast path — catch early non-FILL before paying
        // the trait / slice-construction overhead.  Mirrors the
        // shortcut in BitString's impls_for_leading_zeros.rs.
        if !WORD_ALIGNED && start_offset != 0 {
            let first_val = w0 >> start_offset;
            let first_limit = (WORD_BITS - start_offset as usize).min(self.bit_len);
            let first_count = if FILL == 0 {
                first_val.trailing_zeros() as usize
            } else {
                (!first_val).trailing_zeros() as usize
            };
            if first_count < first_limit {
                return first_count;
            }
        } else if w0 != FILL {
            let count = if FILL == 0 {
                w0.trailing_zeros() as usize
            } else {
                (!w0).trailing_zeros() as usize
            };
            return count.min(self.bit_len);
        }

        // Reconstruct slice for the trait call.
        let words = unsafe { core::slice::from_raw_parts(words_ptr, all_words.len() - word_start) };
        words.leading_value_bits::<FILL, WORD_ALIGNED>(start_offset, self.bit_len)
    }
}
