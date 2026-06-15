use super::*;

impl BitString {
    #[inline]
    pub fn new() -> Self {
        Self {
            bits: Vec::new(),
            len: 0,
        }
    }
}

impl Default for BitString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
