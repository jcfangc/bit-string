use alloc::vec::Vec;

use super::*;

impl BitString {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<bool> {
        (index < self.len).then(|| {
            let word = self.bits[index / 64];
            let mask = 1u64 << (index % 64);
            word & mask != 0
        })
    }
}

impl BitString {
    #[inline]
    pub fn any(&self) -> bool {
        self.count_ones() != 0
    }

    #[inline]
    pub fn all(&self) -> bool {
        self.count_ones() == self.len
    }

    #[inline]
    pub fn is_all_zeros(&self) -> bool {
        !self.any()
    }

    #[inline]
    pub fn is_all_ones(&self) -> bool {
        self.all()
    }
}

impl BitString {
    /// Returns the internal little-endian words.
    ///
    /// Bit index `i` is stored in word `i / 64`, bit offset `i % 64`.
    /// Unused high bits in the last word are guaranteed to be zero.
    #[inline]
    pub fn as_words(&self) -> &[u64] {
        &self.bits
    }

    #[inline]
    pub fn first(&self) -> Option<bool> {
        self.get(0)
    }

    #[inline]
    pub fn last(&self) -> Option<bool> {
        self.len.checked_sub(1).and_then(|index| self.get(index))
    }

    #[inline]
    pub fn to_bool_vec(&self) -> Vec<bool> {
        self.iter().collect()
    }
}

#[cfg(test)]
mod tests_for_get;
