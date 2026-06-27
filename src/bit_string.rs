use alloc::vec::Vec;

use crate::WORD_BITS;

#[derive(Clone, PartialEq, Eq)]
pub struct BitString {
    words: Vec<u64>,
    bit_len: usize,
}

impl BitString {
    #[inline]
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// Returns the internal little-endian words.
    ///
    /// Bit index `i` is stored in word `i / 64`, bit offset `i % 64`.
    /// Unused high bits in the last word are guaranteed to be zero.
    #[inline]
    pub fn words(&self) -> &[u64] {
        &self.words
    }

    /// Returns a zero-copy [`BitStr`] view of the entire bit string.
    #[inline]
    pub fn as_bit_str(&self) -> crate::BitStr<'_> {
        crate::BitStr {
            source: self,
            start: 0,
            bit_len: self.bit_len,
        }
    }
}

pub mod errors;
mod impls_for_access;
mod impls_for_bit_arith;
mod impls_for_construction;
mod impls_for_editing;
mod impls_for_fmt;
mod impls_for_hash;
mod impls_for_iter;
mod impls_for_matching;
mod impls_for_ord;
mod impls_for_predicates;

#[cfg(test)]
mod tests_for_proptest;
