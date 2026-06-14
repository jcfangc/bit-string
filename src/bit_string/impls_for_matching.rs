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
    pub fn contains(&self, needle: &Self) -> bool {
        self.find(needle).is_some()
    }

    pub fn find(&self, needle: &Self) -> Option<usize> {
        if needle.len == 0 {
            return Some(0);
        }

        if needle.len > self.len {
            return None;
        }

        let last_start = self.len - needle.len;

        (0..=last_start).find(|&index| bits_equal_at(self, index, needle))
    }

    pub fn rfind(&self, needle: &Self) -> Option<usize> {
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
            .then(|| self.slice_from(prefix.len))
    }

    pub fn strip_suffix(&self, suffix: &Self) -> Option<Self> {
        self.ends_with(suffix)
            .then(|| self.slice_until(self.len - suffix.len))
    }
}

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
