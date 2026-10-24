//! Alignment consts are caller guarantees when `true`; `false` makes no
//! alignment guarantee and retains the general path.

use crate::BitStr;
use crate::traits::*;
use crate::{AnchorHint, ContainsOptions, LocationHint};
use crate::{SMALL_WORDS, WORD_BITS};
use int_intervals::UsizeCO;

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
        debug_assert!(!WORD_ALIGNED || so == 0);
        debug_assert!(!ND_WORD_ALIGNED || needle.start.is_multiple_of(WORD_BITS));
        let needle_first_word = if ND_WORD_ALIGNED {
            needle.source.words()[needle.start / WORD_BITS]
        } else {
            needle.get_chunk(0)
        };
        let needle_words = [needle_first_word];
        let needle_len = needle.bit_len;
        if WORD_ALIGNED || so == 0 {
            return words[sw..].find_first_word(
                self.bit_len,
                &needle_words,
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
                .find_any_candidate(remaining, &needle_words, needle_len, &mut |pos| {
                    self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                })
                .is_some()
        {
            return None;
        }
        aligned
            .find_first_word(remaining, &needle_words, needle_len, &mut |pos| {
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
        debug_assert!(!WORD_ALIGNED || so == 0);
        debug_assert!(!ND_WORD_ALIGNED || needle.start.is_multiple_of(WORD_BITS));
        let needle_first_word = if ND_WORD_ALIGNED {
            needle.source.words()[needle.start / WORD_BITS]
        } else {
            needle.get_chunk(0)
        };
        let needle_words = [needle_first_word];
        let needle_len = needle.bit_len;
        if WORD_ALIGNED || so == 0 {
            return words[sw..].find_last_word(
                self.bit_len,
                &needle_words,
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
                    .find_any_candidate(remaining, &needle_words, needle_len, &mut |pos| {
                        self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                    })
                    .is_some();
            if maybe_candidate {
                if let Some(pos) =
                    aligned.find_last_word(remaining, &needle_words, needle_len, &mut |pos| {
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
        options: ContainsOptions,
    ) -> bool {
        // Keep the established SIMD Any path for the default and prefix-only
        // cases. Explicit location or suffix hints use a directional scan so
        // they can prioritize the caller's expected match region or anchor.
        if options.location() != LocationHint::Unknown || options.anchor() == AnchorHint::Suffix {
            return self.contains_hinted(needle, options);
        }
        if needle.bit_len == 0 {
            return true;
        }
        if needle.bit_len > self.bit_len {
            return false;
        }
        let words = self.source.words();
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        debug_assert!(!HS_WORD_ALIGNED || so == 0);
        debug_assert!(!ND_WORD_ALIGNED || needle.start.is_multiple_of(WORD_BITS));
        let needle_first_word = if ND_WORD_ALIGNED {
            needle.source.words()[needle.start / WORD_BITS]
        } else {
            needle.get_chunk(0)
        };
        let needle_words = [needle_first_word];
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
                .find_any_candidate(remaining, &needle_words, needle_len, &mut |pos| {
                    self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos + first_bits, needle)
                })
                .is_some();
        }
        words[sw..]
            .find_any_candidate(self.bit_len, &needle_words, needle_len, &mut |pos| {
                self.bits_equal_at_inner::<false, ND_WORD_ALIGNED>(pos, needle)
            })
            .is_some()
    }

    fn contains_hinted(&self, needle: BitStr<'_>, options: ContainsOptions) -> bool {
        let needle_len = needle.bit_len();
        if needle_len == 0 {
            return true;
        }
        if needle_len > self.bit_len() {
            return false;
        }

        let anchor_len = needle_len.min(WORD_BITS);
        let anchor_start = match options.anchor() {
            AnchorHint::Prefix => 0,
            AnchorHint::Suffix => needle_len - anchor_len,
            _ => 0,
        };
        let anchor = (options.anchor() != AnchorHint::Unknown).then(|| {
            needle.slice(
                UsizeCO::checked_from_start_len(anchor_start, anchor_len)
                    .expect("anchor interval is within the needle"),
            )
        });
        let last_start = self.bit_len() - needle_len;

        let matches_at = |start: usize| {
            let Some(anchor) = anchor else {
                return self.matches_at_str(start, needle);
            };
            if !self.matches_at_str(start + anchor_start, anchor) {
                return false;
            }
            anchor_len == needle_len || self.matches_at_str(start, needle)
        };
        match options.location() {
            LocationHint::NearEnd => (0..=last_start).rev().any(matches_at),
            _ => (0..=last_start).any(matches_at),
        }
    }
}
