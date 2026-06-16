use super::*;

impl BitString {
    #[inline]
    pub fn new() -> Self {
        Self {
            words: Vec::new(),
            bit_len: 0,
        }
    }
}

impl Default for BitString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
