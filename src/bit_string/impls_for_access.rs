use crate::traits::*;

use super::*;

impl BitString {
    #[inline]
    pub fn get(&self, index: usize) -> Option<bool> {
        (index < self.bit_len).then(|| {
            let word = self.words[index / 64];
            let mask = 1u64 << (index % 64);
            word & mask != 0
        })
    }

    #[inline]
    pub fn first(&self) -> Option<bool> {
        self.get(0)
    }

    #[inline]
    pub fn last(&self) -> Option<bool> {
        self.bit_len
            .checked_sub(1)
            .and_then(|index| self.get(index))
    }

    /// Reads up to 64 bits starting at `bit_start`, returning them in the
    /// low bits of a `u64`.
    ///
    /// Bits beyond `self.len()` are treated as zero.
    #[inline]
    pub fn get_chunk(&self, bit_start: usize) -> u64 {
        self.words.read_word_at(bit_start)
    }
}

#[cfg(test)]
mod tests_for_get;
#[cfg(test)]
mod tests_for_get_chunk;
