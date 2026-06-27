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
        let full_words = self.bit_len / WORD_BITS;
        let rem = self.bit_len % WORD_BITS;

        if self.start % WORD_BITS == 0 {
            // Aligned: hash full words element-by-element (not as a [u64]
            // slice, which would inject a length prefix that the unaligned
            // path doesn't emit).
            let sw = self.start / WORD_BITS;
            for w in &words[sw..][..full_words] {
                w.hash(state);
            }
        } else {
            // Unaligned: each view-word straddles a source-word boundary,
            // so we must go through read_word_at word by word.
            for i in 0..full_words {
                let w = words.read_word_at(self.start + i * WORD_BITS);
                w.hash(state);
            }
        }

        if rem > 0 {
            let tail_start = self.start + full_words * WORD_BITS;
            (words.read_word_at(tail_start) & low_mask(rem)).hash(state);
        }
    }
}

#[cfg(test)]
mod tests_for_hash;
