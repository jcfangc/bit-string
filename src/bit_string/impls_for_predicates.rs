use super::*;

impl BitString {
    #[inline]
    pub fn any(&self) -> bool {
        self.words.iter().any(|&word| word != 0)
    }

    #[inline]
    pub fn all(&self) -> bool {
        let full_word_len = self.bit_len / crate::WORD_BITS;
        if self.words[..full_word_len]
            .iter()
            .any(|&word| word != u64::MAX)
        {
            return false;
        }

        let tail_len = self.bit_len % crate::WORD_BITS;
        tail_len == 0 || self.words[full_word_len] == crate::low_mask(tail_len)
    }

    #[inline]
    pub fn is_all_zeros(&self) -> bool {
        !self.any()
    }

    #[inline]
    pub fn is_all_ones(&self) -> bool {
        self.all()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bit_len == 0
    }
}
