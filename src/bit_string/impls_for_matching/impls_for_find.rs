use crate::SMALL_WORDS;

use super::*;

mod funcs_for_contains_core;
mod funcs_for_find_core;
mod funcs_for_rfind_core;

impl BitString {
    #[inline]
    pub fn contains(&self, needle: &Self) -> bool {
        if needle.bit_len == 0 {
            return true;
        }
        if needle.bit_len > self.bit_len {
            return false;
        }

        funcs_for_contains_core::find_any_candidate(
            &self.words,
            self.bit_len,
            needle.as_words(),
            needle.bit_len,
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
        if self.words.len() >= SMALL_WORDS
            && !funcs_for_contains_core::find_any_candidate(
                &self.words,
                self.bit_len,
                needle.as_words(),
                needle.bit_len,
                &mut |pos| bits_equal_at(self, pos, needle),
            )
            .is_some()
        {
            return None;
        }

        funcs_for_find_core::find_first_word(
            &self.words,
            self.bit_len,
            needle.as_words(),
            needle.bit_len,
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
        if self.words.len() >= SMALL_WORDS
            && !funcs_for_contains_core::find_any_candidate(
                &self.words,
                self.bit_len,
                needle.as_words(),
                needle.bit_len,
                &mut |pos| bits_equal_at(self, pos, needle),
            )
            .is_some()
        {
            return None;
        }

        funcs_for_rfind_core::find_last_word(
            &self.words,
            self.bit_len,
            needle.as_words(),
            needle.bit_len,
            &mut |pos| bits_equal_at(self, pos, needle),
        )
    }
}

#[cfg(test)]
mod tests_for_contains;

#[cfg(test)]
mod tests_for_find;

#[cfg(test)]
mod tests_for_rfind;
