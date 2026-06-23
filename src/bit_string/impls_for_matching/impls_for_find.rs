use crate::SMALL_WORDS;
use crate::traits::*;

use super::*;

impl BitString {
    #[inline]
    pub fn contains(&self, needle: &Self) -> bool {
        if needle.bit_len == 0 {
            return true;
        }
        if needle.bit_len > self.bit_len {
            return false;
        }

        self.words
            .find_any_candidate(self.bit_len, needle.words(), needle.bit_len, &mut |pos| {
                self.bits_equal_at(pos, needle)
            })
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
            && !self
                .words
                .find_any_candidate(self.bit_len, needle.words(), needle.bit_len, &mut |pos| {
                    self.bits_equal_at(pos, needle)
                })
                .is_some()
        {
            return None;
        }

        self.words
            .find_first_word(self.bit_len, needle.words(), needle.bit_len, &mut |pos| {
                self.bits_equal_at(pos, needle)
            })
    }

    pub fn rfind(&self, needle: &Self) -> Option<usize> {
        if needle.bit_len == 0 {
            return Some(self.bit_len);
        }
        if needle.bit_len > self.bit_len {
            return None;
        }
        if self.words.len() >= SMALL_WORDS
            && !self
                .words
                .find_any_candidate(self.bit_len, needle.words(), needle.bit_len, &mut |pos| {
                    self.bits_equal_at(pos, needle)
                })
                .is_some()
        {
            return None;
        }

        self.words
            .find_last_word(self.bit_len, needle.words(), needle.bit_len, &mut |pos| {
                self.bits_equal_at(pos, needle)
            })
    }
}

#[cfg(test)]
mod tests_for_contains;

#[cfg(test)]
mod tests_for_find;

#[cfg(test)]
mod tests_for_rfind;
