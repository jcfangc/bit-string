use crate::funcs_for_bits::*;

use super::*;

mod funcs_for_find_core;

impl BitString {
    #[inline]
    pub fn contains(&self, needle: &Self) -> bool {
        if needle.bit_len == 0 {
            return true;
        }
        if needle.bit_len > self.bit_len {
            return false;
        }

        let needle_words = needle.as_words();
        let needle_first = needle_words[0];
        let needle_mask = low_mask(needle.bit_len.min(WORD_BITS));
        let last_start = self.bit_len - needle.bit_len;

        funcs_for_contains_core::find_first_candidate(
            &self.words,
            needle_first,
            needle_mask,
            last_start,
            &mut |pos| bits_equal_at(self, pos, needle),
        )
        .is_some()
    }

    pub fn find(&self, needle: &Self) -> Option<usize> {
        if needle.bit_len == 0 {
            return Some(0);
        }

        if needle.bit_len > self.bit_len {
            return None;
        }

        let last_start = self.bit_len - needle.bit_len;

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
}

#[cfg(test)]
mod tests_for_contains;

#[cfg(test)]
mod tests_for_find;

#[cfg(test)]
mod tests_for_rfind;
