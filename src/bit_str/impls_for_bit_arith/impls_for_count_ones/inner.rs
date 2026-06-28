use crate::traits::WordsScan;
use crate::{WORD_BITS, low_mask};

use crate::BitStr;

impl<'bs> BitStr<'bs> {
    /// `count_ones` with compile-time alignment signal.
    #[inline]
    pub(crate) fn count_ones_inner<const WORD_ALIGNED: bool>(&self) -> usize {
        if self.bit_len == 0 {
            return 0;
        }
        let words = self.source.words();
        if WORD_ALIGNED || self.start % WORD_BITS == 0 {
            let ws = self.start / WORD_BITS;
            return words[ws..].count_ones(self.bit_len);
        }
        let sw = self.start / WORD_BITS;
        let so = self.start % WORD_BITS;
        let end = self.start + self.bit_len;
        let last_word = (end - 1) / WORD_BITS;
        if sw == last_word {
            let mask = low_mask(self.bit_len) << so;
            return (words[sw] & mask).count_ones() as usize;
        }
        let mut count = 0usize;
        count += (words[sw] >> so).count_ones() as usize;
        let end_rem = end % WORD_BITS;
        let mid_start = sw + 1;
        let mid_end = if end_rem == 0 {
            last_word + 1
        } else {
            last_word
        };
        let fwc = mid_end.saturating_sub(mid_start);
        if fwc > 0 {
            count += words[mid_start..mid_end].count_ones(fwc * WORD_BITS);
        }
        if end_rem != 0 {
            count += (words[last_word] & low_mask(end_rem)).count_ones() as usize;
        }
        count
    }
}
