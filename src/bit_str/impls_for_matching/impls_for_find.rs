use crate::BitString;

use super::*;

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
        let max_pos = self.bit_len - needle.bit_len();
        for pos in 0..=max_pos {
            if self.bits_equal_at(pos, needle) {
                return true;
            }
        }
        false
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
        let max_pos = self.bit_len - needle.bit_len();
        for pos in 0..=max_pos {
            if self.bits_equal_at(pos, needle) {
                return Some(pos);
            }
        }
        None
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
        let max_pos = self.bit_len - needle.bit_len();
        for pos in (0..=max_pos).rev() {
            if self.bits_equal_at(pos, needle) {
                return Some(pos);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests_for_find;
