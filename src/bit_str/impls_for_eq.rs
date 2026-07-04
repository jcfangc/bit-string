use crate::{BitStr, WORD_BITS};

impl PartialEq for BitStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.bit_len != other.bit_len {
            return false;
        }
        if self.bit_len == 0 {
            return true;
        }
        let hs_aligned = self.start % WORD_BITS == 0;
        let nd_aligned = other.start % WORD_BITS == 0;
        match (hs_aligned, nd_aligned) {
            (true, true) => self.bits_equal_at_inner::<true, true>(0, *other),
            (true, false) => self.bits_equal_at_inner::<true, false>(0, *other),
            (false, true) => self.bits_equal_at_inner::<false, true>(0, *other),
            (false, false) => self.bits_equal_at_inner::<false, false>(0, *other),
        }
    }
}

impl Eq for BitStr<'_> {}

#[cfg(test)]
mod tests_for_eq;
