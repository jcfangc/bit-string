use int_interval::UsizeCO;

use super::*;

#[inline]
fn bits_equal_at(haystack: &BitString, offset: usize, needle: &BitString) -> bool {
    for index in 0..needle.len {
        if haystack.get(offset + index).unwrap() != needle.get(index).unwrap() {
            return false;
        }
    }

    true
}

#[inline]
fn slice_from(bit_string: &BitString, start: usize) -> BitString {
    let len = bit_string.len - start;

    if len == 0 {
        return BitString::new();
    }

    let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
    bit_string.slice(interval)
}

#[inline]
fn slice_until(bit_string: &BitString, end: usize) -> BitString {
    if end == 0 {
        return BitString::new();
    }

    let interval = UsizeCO::checked_from_start_len(0, end).unwrap();
    bit_string.slice(interval)
}

impl BitString {
    pub fn matches_at(&self, index: usize, pattern: &Self) -> bool {
        if index > self.len {
            return false;
        }

        if pattern.len > self.len - index {
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
        suffix.len <= self.len && self.matches_at(self.len - suffix.len, suffix)
    }

    #[inline]
    pub fn contains_bits(&self, needle: &Self) -> bool {
        self.find_bits(needle).is_some()
    }

    pub fn find_bits(&self, needle: &Self) -> Option<usize> {
        if needle.len == 0 {
            return Some(0);
        }

        if needle.len > self.len {
            return None;
        }

        let last_start = self.len - needle.len;

        (0..=last_start).find(|&index| bits_equal_at(self, index, needle))
    }

    pub fn rfind_bits(&self, needle: &Self) -> Option<usize> {
        if needle.len == 0 {
            return Some(self.len);
        }

        if needle.len > self.len {
            return None;
        }

        let last_start = self.len - needle.len;

        (0..=last_start)
            .rev()
            .find(|&index| bits_equal_at(self, index, needle))
    }

    pub fn strip_prefix(&self, prefix: &Self) -> Option<Self> {
        self.starts_with(prefix)
            .then(|| slice_from(self, prefix.len))
    }

    pub fn strip_suffix(&self, suffix: &Self) -> Option<Self> {
        self.ends_with(suffix)
            .then(|| slice_until(self, self.len - suffix.len))
    }
}
