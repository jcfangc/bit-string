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

        for i in 0..full_words {
            let w = words.read_word_at(self.start + i * WORD_BITS);
            w.hash(state);
        }

        if rem > 0 {
            let w = words.read_word_at(self.start + full_words * WORD_BITS);
            (w & low_mask(rem)).hash(state);
        }
    }
}

#[cfg(test)]
mod tests_for_hash;
