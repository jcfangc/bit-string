use crate::{WORD_BITS, assert_valid_width};

/// Returns the smallest number of fixed-width codes that occupies whole words.
#[inline]
pub(crate) const fn layout_block_len<const BITS: u8>() -> usize {
    assert_valid_width::<BITS>();
    WORD_BITS / gcd(WORD_BITS, BITS as usize)
}

#[inline]
const fn gcd(mut lhs: usize, mut rhs: usize) -> usize {
    while rhs != 0 {
        let remainder = lhs % rhs;
        lhs = rhs;
        rhs = remainder;
    }
    lhs
}
