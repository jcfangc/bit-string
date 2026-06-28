use crate::BitStr;
use crate::{FILL_ONES, FILL_ZEROS, WORD_BITS};

mod inner;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub fn leading_zeros(&self) -> usize {
        if self.start % WORD_BITS == 0 {
            self.leading_value_bits_inner::<FILL_ZEROS, true>()
        } else {
            self.leading_value_bits_inner::<FILL_ZEROS, false>()
        }
    }
    #[inline]
    pub fn leading_ones(&self) -> usize {
        if self.start % WORD_BITS == 0 {
            self.leading_value_bits_inner::<FILL_ONES, true>()
        } else {
            self.leading_value_bits_inner::<FILL_ONES, false>()
        }
    }
}

#[cfg(test)]
mod tests_for_leading_ones;
#[cfg(test)]
mod tests_for_leading_zeros;
