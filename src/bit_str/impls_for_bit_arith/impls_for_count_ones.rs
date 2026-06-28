use crate::WORD_BITS;

use crate::BitStr;

mod inner;

impl<'bs> BitStr<'bs> {
    /// Returns the number of bits set to 1.
    #[inline]
    pub fn count_ones(&self) -> usize {
        if self.start % WORD_BITS == 0 {
            self.count_ones_inner::<true>()
        } else {
            self.count_ones_inner::<false>()
        }
    }

    /// Returns the number of bits set to 0.
    #[inline]
    pub fn count_zeros(&self) -> usize {
        self.bit_len - self.count_ones()
    }
}

#[cfg(test)]
mod tests_for_count_ones;
