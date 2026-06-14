use super::*;
use alloc::vec::Vec;

impl BitString {
    #[inline]
    pub fn new() -> Self {
        Self {
            bits: Vec::new().into_boxed_slice(),
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
