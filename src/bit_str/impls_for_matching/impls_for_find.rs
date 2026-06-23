use crate::BitString;
use crate::SMALL_WORDS;
use crate::WORD_BITS;
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

        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;

        // Unaligned start: check the first partial word before delegating to
        // SIMD on the aligned remainder.
        if so != 0 {
            let first_bits = (WORD_BITS - so).min(self.bit_len);
            let max = first_bits.saturating_sub(needle.bit_len());
            for p in 0..=max {
                if self.bits_equal_at(p, needle) {
                    return true;
                }
            }
            let remaining = self.bit_len - first_bits;
            if remaining == 0 {
                return false;
            }
            // Aligned remainder — SIMD shift-outer candidate scan.
            let aligned = &words[sw + 1..];
            return aligned
                .find_any_candidate(remaining, needle.words(), needle.bit_len(), &mut |pos| {
                    self.bits_equal_at(pos + first_bits, needle)
                })
                .is_some();
        }

        // Word-aligned: full SIMD on the relevant suffix.
        words[sw..]
            .find_any_candidate(self.bit_len, needle.words(), needle.bit_len(), &mut |pos| {
                self.bits_equal_at(pos, needle)
            })
            .is_some()
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

        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        let needle_words = needle.words();
        let needle_len = needle.bit_len();

        // Word-aligned fast path: the entire view is SIMD-friendly.
        if so == 0 {
            return words[sw..].find_first_word(
                self.bit_len,
                needle_words,
                needle_len,
                &mut |pos| self.bits_equal_at(pos, needle),
            );
        }

        // Unaligned: scan the first partial word, then SIMD for the rest.
        let first_bits = (WORD_BITS - so).min(self.bit_len);
        let max = first_bits.saturating_sub(needle_len);
        for p in 0..=max {
            if self.bits_equal_at(p, needle) {
                return Some(p);
            }
        }

        let remaining = self.bit_len - first_bits;
        if remaining == 0 {
            return None;
        }

        let aligned = &words[sw + 1..];

        // Quick rejection before the more expensive word-outer scan.
        if aligned.len() >= SMALL_WORDS
            && !aligned
                .find_any_candidate(remaining, needle_words, needle_len, &mut |pos| {
                    self.bits_equal_at(pos + first_bits, needle)
                })
                .is_some()
        {
            return None;
        }

        aligned
            .find_first_word(remaining, needle_words, needle_len, &mut |pos| {
                self.bits_equal_at(pos + first_bits, needle)
            })
            .map(|pos| pos + first_bits)
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

        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        let needle_words = needle.words();
        let needle_len = needle.bit_len();

        // Word-aligned fast path.
        if so == 0 {
            return words[sw..].find_last_word(
                self.bit_len,
                needle_words,
                needle_len,
                &mut |pos| self.bits_equal_at(pos, needle),
            );
        }

        // Unaligned: SIMD on the aligned remainder first (reverse order),
        // then fall back to the first partial word.
        let first_bits = (WORD_BITS - so).min(self.bit_len);
        let remaining = self.bit_len - first_bits;

        if remaining > 0 {
            let aligned = &words[sw + 1..];

            if aligned.len() >= SMALL_WORDS
                && aligned
                    .find_any_candidate(remaining, needle_words, needle_len, &mut |pos| {
                        self.bits_equal_at(pos + first_bits, needle)
                    })
                    .is_some()
            {
                if let Some(pos) =
                    aligned.find_last_word(remaining, needle_words, needle_len, &mut |pos| {
                        self.bits_equal_at(pos + first_bits, needle)
                    })
                {
                    return Some(pos + first_bits);
                }
            }
        }

        // Check the first partial word.
        let max = first_bits.saturating_sub(needle_len);
        for p in (0..=max).rev() {
            if self.bits_equal_at(p, needle) {
                return Some(p);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests_for_find;
