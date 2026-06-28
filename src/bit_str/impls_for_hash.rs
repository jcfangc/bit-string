use core::hash::{Hash, Hasher};

use crate::traits::*;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Hash with compile-time alignment signal.
    #[inline]
    pub(crate) fn hash_inner<const WORD_ALIGNED: bool, H: Hasher>(&self, state: &mut H) {
        self.bit_len.hash(state);
        if self.bit_len == 0 {
            return;
        }

        let words = self.source.words();
        let full_words = self.bit_len / WORD_BITS;
        let rem = self.bit_len % WORD_BITS;

        if WORD_ALIGNED || self.start % WORD_BITS == 0 {
            let sw = self.start / WORD_BITS;
            for w in &words[sw..][..full_words] {
                w.hash(state);
            }
        } else {
            for i in 0..full_words {
                words.read_word_at(self.start + i * WORD_BITS).hash(state);
            }
        }

        if rem > 0 {
            let tail_start = self.start + full_words * WORD_BITS;
            (words.read_word_at(tail_start) & low_mask(rem)).hash(state);
        }
    }
}

impl Hash for BitStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.start % WORD_BITS == 0 {
            self.hash_inner::<true, H>(state)
        } else {
            self.hash_inner::<false, H>(state)
        }
    }
}

#[cfg(test)]
mod tests_for_hash;
