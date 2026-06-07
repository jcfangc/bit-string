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
