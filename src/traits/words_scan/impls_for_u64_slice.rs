use super::WordsScan;
use super::funcs_for_count_ones;
use super::funcs_for_ends;

impl WordsScan for [u64] {
    #[inline]
    fn count_ones(&self, bit_len: usize) -> usize {
        funcs_for_count_ones::count_ones(self, bit_len)
    }

    #[inline(always)]
    fn leading_value_bits<const FILL: u64, const WORD_ALIGNED: bool>(
        &self,
        start_offset: u32,
        bit_len: usize,
    ) -> usize {
        funcs_for_ends::leading::<FILL, WORD_ALIGNED>(self, start_offset, bit_len)
    }

    #[inline]
    fn trailing_value_bits<const FILL: u64, const WORD_ALIGNED: bool>(
        &self,
        start_offset: u32,
        bit_len: usize,
    ) -> usize {
        funcs_for_ends::trailing::<FILL, WORD_ALIGNED>(self, start_offset, bit_len)
    }
}
