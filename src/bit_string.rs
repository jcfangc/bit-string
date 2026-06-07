use alloc::boxed::Box;

pub struct BitString {
    bits: Box<[u64]>,
    len: usize,
}

pub mod errors;
mod impls_for_access;
mod impls_for_construction;
