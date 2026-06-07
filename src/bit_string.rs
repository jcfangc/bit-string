use alloc::boxed::Box;

const WORD_BITS: usize = u64::BITS as usize;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BitString {
    bits: Box<[u64]>,
    len: usize,
}

pub mod errors;
mod impls_for_access;
mod impls_for_bit_ops;
mod impls_for_construction;
mod impls_for_editing;
mod impls_for_fmt;
mod impls_for_iter;

pub(crate) mod funcs_for_share;
