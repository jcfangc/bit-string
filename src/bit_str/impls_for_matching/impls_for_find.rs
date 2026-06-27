use crate::traits::*;
use crate::{SMALL_WORDS, WORD_BITS};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// Returns `true` if `needle` is contained within `self`.
    #[inline]
    pub fn contains(&self, needle: BitStr<'_>) -> bool {
        if needle.bit_len == 0 {
            return true;
        }
        if needle.bit_len > self.bit_len {
            return false;
        }

        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        let needle_words = needle.source.words();
        let needle_len = needle.bit_len;

        // Unaligned start: check the first partial word before delegating to
        // SIMD on the aligned remainder.
        if so != 0 {
            let first_bits = (WORD_BITS - so).min(self.bit_len);
            let max = first_bits.min(self.bit_len.saturating_sub(needle_len));
            for p in 0..=max {
                if self.bits_equal_at(p, needle) {
                    return true;
                }
            }
            let remaining = self.bit_len - first_bits;
            if remaining == 0 {
                return false;
            }
            let aligned = &words[sw + 1..];
            return aligned
                .find_any_candidate(remaining, needle_words, needle_len, &mut |pos| {
                    self.bits_equal_at(pos + first_bits, needle)
                })
                .is_some();
        }

        // Word-aligned: full SIMD on the relevant suffix.
        words[sw..]
            .find_any_candidate(self.bit_len, needle_words, needle_len, &mut |pos| {
                self.bits_equal_at(pos, needle)
            })
            .is_some()
    }

    /// Returns the index of the first occurrence of `needle`, or `None`.
    #[inline]
    pub fn find(&self, needle: BitStr<'_>) -> Option<usize> {
        if needle.bit_len == 0 {
            return Some(0);
        }
        if needle.bit_len > self.bit_len {
            return None;
        }

        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        let needle_words = needle.source.words();
        let needle_len = needle.bit_len;

        // Word-aligned fast path.
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
        let max = first_bits.min(self.bit_len.saturating_sub(needle_len));
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
    pub fn rfind(&self, needle: BitStr<'_>) -> Option<usize> {
        if needle.bit_len == 0 {
            return Some(self.bit_len);
        }
        if needle.bit_len > self.bit_len {
            return None;
        }

        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        let needle_words = needle.source.words();
        let needle_len = needle.bit_len;

        // Word-aligned fast path.
        if so == 0 {
            return words[sw..].find_last_word(
                self.bit_len,
                needle_words,
                needle_len,
                &mut |pos| self.bits_equal_at(pos, needle),
            );
        }

        // Unaligned: SIMD on the aligned remainder first (reverse),
        // then fall back to the first partial word.
        let first_bits = (WORD_BITS - so).min(self.bit_len);
        let remaining = self.bit_len - first_bits;

        if remaining > 0 {
            let aligned = &words[sw + 1..];

            // Quick rejection via SIMD candidate scan — only when the
            // aligned portion is large enough to amortize setup cost.
            // When it's too small we skip the gate and scan directly
            // (find_last_word has its own scalar fallback).
            let maybe_candidate = aligned.len() < SMALL_WORDS
                || aligned
                    .find_any_candidate(remaining, needle_words, needle_len, &mut |pos| {
                        self.bits_equal_at(pos + first_bits, needle)
                    })
                    .is_some();

            if maybe_candidate {
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
        let max = first_bits.min(self.bit_len.saturating_sub(needle_len));
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
