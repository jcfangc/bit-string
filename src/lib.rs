#![no_std]

extern crate alloc;

pub(crate) const WORD_BITS: usize = u64::BITS as usize;

pub(crate) mod funcs_for_bits;
pub(crate) use funcs_for_bits::*;

mod bit_string;

pub use bit_string::BitString;

pub use bit_string::errors::*;
