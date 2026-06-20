use alloc::vec::Vec;

use crate::WORD_BITS;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BitString {
    words: Vec<u64>,
    bit_len: usize,
}

pub mod errors;
mod impls_for_access;
mod impls_for_bit_arith;
mod impls_for_construction;
mod impls_for_editing;
mod impls_for_fmt;
mod impls_for_iter;
mod impls_for_matching;

pub(crate) mod traits;

#[cfg(test)]
mod tests_for_proptest;
