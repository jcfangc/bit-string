use core::hash::{Hash, Hasher};

use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

impl Hash for BitStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bit_len.hash(state);
        if self.bit_len == 0 {
            return;
        }

        let words = self.source.words();
        let (sw, s) = (self.start / WORD_BITS, self.start % WORD_BITS);

        if s == 0 {
            // Fully word-aligned: bulk-hash full words as a slice.
            let full_words = self.bit_len / WORD_BITS;
            words[sw..][..full_words].hash(state);
        } else {
            // First partial word from the source word at `sw`.
            let first_len = (WORD_BITS - s).min(self.bit_len);
            (words.read_word_at(self.start) & low_mask(first_len)).hash(state);

            let remaining = self.bit_len - first_len;
            let mid_words = remaining / WORD_BITS;
            if mid_words > 0 {
                // Middle words are aligned — hash as a contiguous slice.
                words[sw + 1..][..mid_words].hash(state);
            }
        }

        // Tail partial word (always unaligned within the source).
        let rem = self.bit_len % WORD_BITS;
        if rem > 0 {
            let tail_start = self.start + (self.bit_len / WORD_BITS) * WORD_BITS;
            (words.read_word_at(tail_start) & low_mask(rem)).hash(state);
        }
    }
}

#[cfg(test)]
mod tests_for_hash;
