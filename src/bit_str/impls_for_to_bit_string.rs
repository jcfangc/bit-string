use int_interval::UsizeCO;

use crate::{BitStr, BitString};

impl BitStr<'_> {
    /// Copies the bits in this view into a new owned [`BitString`].
    ///
    /// Delegates to [`BitString::slice`] which performs a word-level
    /// copy from the source.
    #[inline]
    pub fn to_bit_string(&self) -> BitString {
        if self.bit_len == 0 {
            return BitString::new();
        }
        self.source
            .slice(UsizeCO::try_new(self.start, self.start + self.bit_len).unwrap())
    }
}

#[cfg(test)]
mod tests_for_to_bit_string;
