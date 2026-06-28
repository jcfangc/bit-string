use alloc::vec::Vec;

/// SIMD-accelerated word-level operations on `[u64]` backing storage.
///
/// Owned methods allocate a new `Vec<u64>` and return it. Assign methods
/// operate in place.
pub(crate) trait BitsArith {
    /// Returns `self & rhs`, allocating a new result.
    fn and(&self, rhs: &[u64]) -> Vec<u64>;
    /// Performs `self &= rhs` in place.
    fn and_assign(&mut self, rhs: &[u64]);

    /// Returns `self | rhs`, allocating a new result.
    fn or(&self, rhs: &[u64]) -> Vec<u64>;
    /// Performs `self |= rhs` in place.
    fn or_assign(&mut self, rhs: &[u64]);

    /// Returns `self ^ rhs`, allocating a new result.
    fn xor(&self, rhs: &[u64]) -> Vec<u64>;
    /// Performs `self ^= rhs` in place.
    fn xor_assign(&mut self, rhs: &[u64]);

    /// Returns `!self`, allocating a new result.
    ///
    /// `bit_len` is used to mask unused high bits in the last word.
    fn not(&self, bit_len: usize) -> Vec<u64>;
    /// Performs `self = !self` in place, masking with `bit_len`.
    fn not_assign(&mut self, bit_len: usize);

    /// Returns `self << amount` (zero-fill), allocating a new result.
    ///
    /// `bit_len` is used to mask unused high bits in the last word.
    fn shl(&self, bit_len: usize, amount: usize) -> Vec<u64>;
    /// Performs `self <<= amount` (zero-fill) in place, masking with
    /// `bit_len`.
    fn shl_assign(&mut self, bit_len: usize, amount: usize);

    /// Returns `self >> amount` (zero-fill), allocating a new result.
    ///
    /// `bit_len` is used to mask unused high bits in the last word.
    fn shr(&self, bit_len: usize, amount: usize) -> Vec<u64>;
    /// Performs `self >>= amount` (zero-fill) in place, masking with
    /// `bit_len`.
    fn shr_assign(&mut self, bit_len: usize, amount: usize);

    /// Returns the number of ones (set bits) in the first `bit_len` bits.
    ///
    /// Bits beyond `bit_len` are assumed to already be zero (masked by
    /// prior calls to [`BitsEdit::mask_unused_bits`]).
    fn count_ones(&self, bit_len: usize) -> usize;

    /// Returns the number of consecutive zero words at the start of `self`.
    ///
    /// All words up to (but not including) the returned index are zero.  If
    /// the return value equals `self.len()`, every word is zero.
    fn leading_zero_words(&self) -> usize;

    /// Returns the number of consecutive all-ones words at the start of
    /// `self`.
    ///
    /// All words up to (but not including) the returned index are all-ones.
    /// If the return value equals `self.len()`, every word is all-ones.
    fn leading_one_words(&self) -> usize;

    /// Returns the number of consecutive zero words at the **end** of `self`.
    ///
    /// All words from `self.len() - count` onwards are zero.
    fn trailing_zero_words(&self) -> usize;

    /// Returns the number of consecutive all-ones words at the **end** of
    /// `self`.
    ///
    /// All words from `self.len() - count` onwards are all-ones.
    fn trailing_one_words(&self) -> usize;
}

pub(crate) mod funcs_for_binary_core;
pub(crate) mod funcs_for_count_ones;
pub(crate) mod funcs_for_not_core;
pub(crate) mod funcs_for_shl_core;
pub(crate) mod funcs_for_shr_core;
pub(crate) mod funcs_for_value_words_core;
pub(crate) mod impls_for_u64_slice;
