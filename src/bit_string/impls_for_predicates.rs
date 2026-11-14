use super::*;

const PREDICATE_WORD_GROUP_LEN: usize = 8;

impl BitString {
    #[inline]
    pub fn any(&self) -> bool {
        let full_word_len = self.bit_len / crate::WORD_BITS;
        let mut word_index = 0;

        while word_index + PREDICATE_WORD_GROUP_LEN <= full_word_len {
            let mut combined = 0;
            for &word in &self.words[word_index..word_index + PREDICATE_WORD_GROUP_LEN] {
                combined |= word;
            }
            if combined != 0 {
                return true;
            }
            word_index += PREDICATE_WORD_GROUP_LEN;
        }

        if self.words[word_index..full_word_len]
            .iter()
            .any(|&word| word != 0)
        {
            return true;
        }

        let tail_len = self.bit_len % crate::WORD_BITS;
        tail_len != 0 && self.words[full_word_len] & crate::low_mask(tail_len) != 0
    }

    #[inline]
    pub fn all(&self) -> bool {
        let full_word_len = self.bit_len / crate::WORD_BITS;
        let mut word_index = 0;

        while word_index + PREDICATE_WORD_GROUP_LEN <= full_word_len {
            let mut combined = u64::MAX;
            for &word in &self.words[word_index..word_index + PREDICATE_WORD_GROUP_LEN] {
                combined &= word;
            }
            if combined != u64::MAX {
                return false;
            }
            word_index += PREDICATE_WORD_GROUP_LEN;
        }

        if self.words[word_index..full_word_len]
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
