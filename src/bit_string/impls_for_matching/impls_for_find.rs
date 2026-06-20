use crate::WORD_BITS;
use crate::funcs_for_bits::low_mask;

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

        let last_start = self.bit_len - needle.bit_len;
        let needle_words = needle.as_words();
        let needle_first = needle_words[0];
        let needle_mask = low_mask(needle.bit_len.min(WORD_BITS));

        // Word-outer, shift-inner — guarantees earliest match.
        for i in 0..self.words.len() {
            let w0 = self.words[i];
            let w1 = self.words.get(i + 1).copied().unwrap_or(0);
            for shift in 0..WORD_BITS {
                let pos = i * WORD_BITS + shift;
                if pos > last_start {
                    break;
                }
                let window = if shift == 0 {
                    w0
                } else {
                    (w0 >> shift) | (w1 << (WORD_BITS - shift))
                };
                if (window & needle_mask) == needle_first && bits_equal_at(self, pos, needle) {
                    return Some(pos);
                }
            }
        }

        None
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
