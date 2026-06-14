use super::*;
use crate::WORD_BITS;
use alloc::vec::Vec;

impl BitString {
    pub(crate) fn from_bool_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        let mut bits = Vec::<u64>::new();
        let mut len = 0usize;

        for value in iter {
            if len % WORD_BITS == 0 {
                bits.push(0);
            }

            if value {
                let word = len / WORD_BITS;
                let offset = len % WORD_BITS;
                bits[word] |= 1u64 << offset;
            }

            len += 1;
        }

        Self {
            bits: bits.into_boxed_slice(),
            len,
        }
    }
}

impl FromIterator<bool> for BitString {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        Self::from_bool_iter(iter)
    }
}

#[cfg(test)]
mod tests_for_from_bool_iter;
