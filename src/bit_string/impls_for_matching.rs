use crate::bit_string::traits::*;
use crate::funcs_for_bits::*;

use super::*;

/// Compare `needle` bits against `haystack` starting at `offset`, using
/// word-level reads so that each iteration compares up to 64 bits.
#[inline]
fn bits_equal_at(haystack: &BitString, offset: usize, needle: &BitString) -> bool {
    let needle_bits = needle.bit_len;
    let needle_words = needle.as_words();
    let full_words = needle_bits / WORD_BITS;
    let rem_bits = needle_bits % WORD_BITS;

    // Full u64 words — needle is always word-aligned at index 0 so we
    // compare needle_words[i] directly.
    for i in 0..full_words {
        let h = haystack.words.read_word_at(offset + i * WORD_BITS);
        if h != needle_words[i] {
            return false;
        }
    }

    // Last partial word (if any).
    if rem_bits > 0 {
        let mask = low_mask(rem_bits);
        let h = haystack.words.read_word_at(offset + full_words * WORD_BITS);
        if (h & mask) != (needle_words[full_words] & mask) {
            return false;
        }
    }

    true
}

impl BitString {
    pub fn matches_at(&self, index: usize, pattern: &Self) -> bool {
        if index > self.bit_len {
            return false;
        }

        if pattern.bit_len > self.bit_len - index {
            return false;
        }

        bits_equal_at(self, index, pattern)
    }

    #[inline]
    pub fn starts_with(&self, prefix: &Self) -> bool {
        self.matches_at(0, prefix)
    }

    #[inline]
    pub fn ends_with(&self, suffix: &Self) -> bool {
        suffix.bit_len <= self.bit_len && self.matches_at(self.bit_len - suffix.bit_len, suffix)
    }

    #[inline]
    pub fn contains(&self, needle: &Self) -> bool {
        self.find(needle).is_some()
    }

    pub fn find(&self, needle: &Self) -> Option<usize> {
        if needle.bit_len == 0 {
            return Some(0);
        }

        if needle.bit_len > self.bit_len {
            return None;
        }

        let last_start = self.bit_len - needle.bit_len;

        // When the needle is very short (≤ 64 bits) we can use a one-word
        // pre-filter: only call the full bits_equal_at when the first
        // 64-bit window matches.
        let needle_words = needle.as_words();
        let needle_first = needle_words[0];
        let needle_mask = low_mask(needle.bit_len.min(WORD_BITS));

        funcs_for_find_core::find_first_word(
            &self.words,
            self.bit_len,
            needle_first,
            needle_mask,
            last_start,
            &mut |pos| bits_equal_at(self, pos, needle),
        )
    }

    pub fn rfind(&self, needle: &Self) -> Option<usize> {
        if needle.bit_len == 0 {
            return Some(self.bit_len);
        }

        if needle.bit_len > self.bit_len {
            return None;
        }

        let last_start = self.bit_len - needle.bit_len;

        (0..=last_start)
            .rev()
            .find(|&index| bits_equal_at(self, index, needle))
    }

    pub fn strip_prefix(&self, prefix: &Self) -> Option<Self> {
        self.starts_with(prefix)
            .then(|| self.slice_from(prefix.bit_len))
    }

    pub fn strip_suffix(&self, suffix: &Self) -> Option<Self> {
        self.ends_with(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }
}

mod funcs_for_find_core;

#[cfg(test)]
mod tests_for_bits_equal_at;

#[cfg(test)]
mod tests_for_matches_at;

#[cfg(test)]
mod tests_for_starts_with;

#[cfg(test)]
mod tests_for_ends_with;

#[cfg(test)]
mod tests_for_contains;

#[cfg(test)]
mod tests_for_find;

#[cfg(test)]
mod tests_for_rfind;

#[cfg(test)]
mod tests_for_strip_prefix;

#[cfg(test)]
mod tests_for_strip_suffix;
