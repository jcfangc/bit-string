use crate::BitString;
use crate::SMALL_WORDS;
use crate::traits::*;

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Returns `true` if `needle` is contained within `self`.
    #[inline]
    pub fn contains(&self, needle: &BitString) -> bool {
        if needle.bit_len() == 0 {
            return true;
        }
        if needle.bit_len() > self.bit_len {
            return false;
        }
        self.find(needle).is_some()
    }

    /// Returns the index of the first occurrence of `needle`, or `None`.
    #[inline]
    pub fn find(&self, needle: &BitString) -> Option<usize> {
        if needle.bit_len() == 0 {
            return Some(0);
        }
        if needle.bit_len() > self.bit_len {
            return None;
        }

        let start_word = self.start / crate::WORD_BITS;
        let start_offset = self.start % crate::WORD_BITS;
        let view_bits = start_offset + self.bit_len;
        let words = &self.source.words()[start_word..];
        let needle_words = needle.words();
        let needle_len = needle.bit_len();

        // Quick rejection: no candidate at all.
        if words.len() >= SMALL_WORDS
            && !words
                .find_any_candidate(view_bits, needle_words, needle_len, &mut |pos| {
                    pos >= start_offset
                        && pos + needle_len <= view_bits
                        && self.bits_equal_at(pos - start_offset, needle)
                })
                .is_some()
        {
            return None;
        }

        // Fine-grained word-outer search for the earliest match.
        words
            .find_first_word(view_bits, needle_words, needle_len, &mut |pos| {
                pos >= start_offset
                    && pos + needle_len <= view_bits
                    && self.bits_equal_at(pos - start_offset, needle)
            })
            .map(|pos| pos - start_offset)
    }

    /// Returns the index of the last occurrence of `needle`, or `None`.
    #[inline]
    pub fn rfind(&self, needle: &BitString) -> Option<usize> {
        if needle.bit_len() == 0 {
            return Some(self.bit_len);
        }
        if needle.bit_len() > self.bit_len {
            return None;
        }

        let start_word = self.start / crate::WORD_BITS;
        let start_offset = self.start % crate::WORD_BITS;
        let view_bits = start_offset + self.bit_len;
        let words = &self.source.words()[start_word..];
        let needle_words = needle.words();
        let needle_len = needle.bit_len();

        // Quick rejection.
        if words.len() >= SMALL_WORDS
            && !words
                .find_any_candidate(view_bits, needle_words, needle_len, &mut |pos| {
                    pos >= start_offset
                        && pos + needle_len <= view_bits
                        && self.bits_equal_at(pos - start_offset, needle)
                })
                .is_some()
        {
            return None;
        }

        // Reverse search for the rightmost match.
        words
            .find_last_word(view_bits, needle_words, needle_len, &mut |pos| {
                pos >= start_offset
                    && pos + needle_len <= view_bits
                    && self.bits_equal_at(pos - start_offset, needle)
            })
            .map(|pos| pos - start_offset)
    }
}

#[cfg(test)]
mod tests_for_find;
