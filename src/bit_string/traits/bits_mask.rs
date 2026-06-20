use crate::WORD_BITS;

/// Bit-mask factory functions for `[u64]` backing storage.
///
/// All methods are associated functions (no `self` receiver) and carry
/// default implementations, so implementors need not override them.
pub(crate) trait BitsMask {
    /// Returns `u64::MAX` when `bits >= WORD_BITS`, otherwise the low `bits`
    /// ones.
    #[inline]
    fn low_mask(bits: usize) -> u64 {
        if bits >= WORD_BITS {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        }
    }

    /// Returns the mask for the last word of a bit string of total length
    /// `len`.
    ///
    /// The number of valid bits in the last word is `len % WORD_BITS`. When
    /// that remainder is zero the last word is full and `u64::MAX` is
    /// returned; otherwise only the low `len % WORD_BITS` bits are set.
    #[inline]
    fn last_word_mask(len: usize) -> u64 {
        let rem = len % WORD_BITS;
        if rem == 0 {
            u64::MAX
        } else {
            (1u64 << rem) - 1
        }
    }
}

impl BitsMask for [u64] {}

#[cfg(test)]
mod tests_for_low_mask;

#[cfg(test)]
mod tests_for_last_word_mask;
