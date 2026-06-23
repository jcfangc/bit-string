use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Returns the number of bits set to 1.
    #[inline]
    pub fn count_ones(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }

        let words = self.source.words();

        // Fast path: when `start` is word-aligned we can delegate directly to
        // the SIMD-accelerated `[u64]::count_ones` on the relevant suffix.
        if self.start % WORD_BITS == 0 {
            let word_start = self.start / WORD_BITS;
            return words[word_start..].count_ones(self.bit_len);
        }

        // Unaligned start: count across word boundaries.
        let start_word = self.start / WORD_BITS;
        let start_offset = self.start % WORD_BITS;
        let end = self.start + self.bit_len;
        let last_word = (end - 1) / WORD_BITS;

        // All bits lie within a single word.
        if start_word == last_word {
            let mask = low_mask(self.bit_len) << start_offset;
            return (words[start_word] & mask).count_ones() as usize;
        }

        let mut count = 0usize;

        // First word: bits from start_offset upward.
        count += (words[start_word] >> start_offset).count_ones() as usize;

        let end_rem = end % WORD_BITS;

        // Middle words: full u64 words, SIMD-accelerated via
        // `[u64]::count_ones`. The last word is included in the SIMD path
        // when it is full (end_rem == 0), otherwise handled separately.
        let mid_start = start_word + 1;
        let mid_end = if end_rem == 0 {
            last_word + 1
        } else {
            last_word
        };
        let full_word_count = mid_end.saturating_sub(mid_start);
        if full_word_count > 0 {
            count += words[mid_start..mid_end].count_ones(full_word_count * WORD_BITS);
        }

        // Last word: partial.
        if end_rem != 0 {
            count += (words[last_word] & low_mask(end_rem)).count_ones() as usize;
        }

        count
    }

    /// Returns the number of bits set to 0.
    #[inline]
    pub fn count_zeros(&self) -> usize {
        self.bit_len - self.count_ones()
    }
}

#[cfg(test)]
mod tests_for_count_ones;
