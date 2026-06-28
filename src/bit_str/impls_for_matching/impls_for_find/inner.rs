use crate::BitStr;
use crate::traits::*;
use crate::{SMALL_WORDS, WORD_BITS};

impl<'bs> BitStr<'bs> {
    #[inline]
    pub(crate) fn find_inner<const WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        needle: BitStr<'_>,
    ) -> Option<usize> {
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
        if WORD_ALIGNED || so == 0 {
            return words[sw..].find_first_word(
                self.bit_len,
                needle_words,
                needle_len,
                &mut |pos| self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos, needle),
            );
        }
        let first_bits = (WORD_BITS - so).min(self.bit_len);
        let max = first_bits.min(self.bit_len.saturating_sub(needle_len));
        for p in 0..=max {
            if self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(p, needle) {
                return Some(p);
            }
        }
        let remaining = self.bit_len - first_bits;
        if remaining == 0 {
            return None;
        }
        let aligned = &words[sw + 1..];
        if aligned.len() >= SMALL_WORDS
            && !aligned
                .find_any_candidate(remaining, needle_words, needle_len, &mut |pos| {
                    self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                })
                .is_some()
        {
            return None;
        }
        aligned
            .find_first_word(remaining, needle_words, needle_len, &mut |pos| {
                self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
            })
            .map(|pos| pos + first_bits)
    }

    #[inline]
    pub(crate) fn rfind_inner<const WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        needle: BitStr<'_>,
    ) -> Option<usize> {
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
        if WORD_ALIGNED || so == 0 {
            return words[sw..].find_last_word(
                self.bit_len,
                needle_words,
                needle_len,
                &mut |pos| self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos, needle),
            );
        }
        let first_bits = (WORD_BITS - so).min(self.bit_len);
        let remaining = self.bit_len - first_bits;
        if remaining > 0 {
            let aligned = &words[sw + 1..];
            let maybe_candidate = aligned.len() < SMALL_WORDS
                || aligned
                    .find_any_candidate(remaining, needle_words, needle_len, &mut |pos| {
                        self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                    })
                    .is_some();
            if maybe_candidate {
                if let Some(pos) =
                    aligned.find_last_word(remaining, needle_words, needle_len, &mut |pos| {
                        self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                    })
                {
                    return Some(pos + first_bits);
                }
            }
        }
        let max = first_bits.min(self.bit_len.saturating_sub(needle_len));
        for p in (0..=max).rev() {
            if self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(p, needle) {
                return Some(p);
            }
        }
        None
    }

    #[inline]
    pub(crate) fn contains_inner<const HS_WORD_ALIGNED: bool, const ND_WORD_ALIGNED: bool>(
        &self,
        needle: BitStr<'_>,
    ) -> bool {
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
        if !HS_WORD_ALIGNED && so != 0 {
            let first_bits = (WORD_BITS - so).min(self.bit_len);
            let max = first_bits.min(self.bit_len.saturating_sub(needle_len));
            for p in 0..=max {
                if self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(p, needle) {
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
                    self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                })
                .is_some();
        }
        words[sw..]
            .find_any_candidate(self.bit_len, needle_words, needle_len, &mut |pos| {
                self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos, needle)
            })
            .is_some()
    }
}
