use super::*;

impl BitString {
    pub fn strip_prefix(&self, prefix: &Self) -> Option<Self> {
        self.starts_with(prefix)
            .then(|| self.slice_from(prefix.bit_len))
    }

    pub fn strip_suffix(&self, suffix: &Self) -> Option<Self> {
        self.ends_with(suffix)
            .then(|| self.slice_until(self.bit_len - suffix.bit_len))
    }
}

#[cfg(test)]
mod tests_for_strip_prefix;

#[cfg(test)]
mod tests_for_strip_suffix;
