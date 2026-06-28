use crate::BitStr;
use crate::WORD_BITS;
use crate::traits::WordsScan;

impl<'bs> BitStr<'bs> {
    #[inline]
    pub(crate) fn leading_value_bits_inner<const FILL: u64, const WORD_ALIGNED: bool>(
        &self,
    ) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        let words = &self.source.words()[self.start / WORD_BITS..];
        let start_offset = (self.start % WORD_BITS) as u32;
        words.leading_value_bits::<FILL, WORD_ALIGNED>(start_offset, self.bit_len)
    }
}
