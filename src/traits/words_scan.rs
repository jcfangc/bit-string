/// Bit-counting and scanning operations on `[u64]` backing storage.
pub(crate) trait WordsScan {
    /// Returns the number of ones (set bits) in the first `bit_len` bits.
    fn count_ones(&self, bit_len: usize) -> usize;

    /// Returns the count of consecutive leading bits equal to `FILL`.
    ///
    /// `self` is pre-trimmed to `words[physical_start / WORD_BITS..]`.
    /// `start_offset` is `physical_start % WORD_BITS`.
    /// When `WORD_ALIGNED` is `true`, `start_offset` is guaranteed to be 0
    /// and the first-word phase is eliminated at compile time.
    fn leading_value_bits<const FILL: u64, const WORD_ALIGNED: bool>(
        &self,
        start_offset: u32,
        bit_len: usize,
    ) -> usize;

    /// Returns the count of consecutive trailing bits equal to `FILL`.
    ///
    /// Same preconditions as [`leading_value_bits`](Self::leading_value_bits).
    fn trailing_value_bits<const FILL: u64, const WORD_ALIGNED: bool>(
        &self,
        start_offset: u32,
        bit_len: usize,
    ) -> usize;
}

pub(crate) mod funcs_for_count_ones;
pub(crate) mod funcs_for_ends;
pub(crate) mod impls_for_u64_slice;
