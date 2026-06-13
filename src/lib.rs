#![no_std]

extern crate alloc;

pub(crate) const WORD_BITS: usize = u64::BITS as usize;

mod bit_string;

pub use bit_string::BitString;

pub use bit_string::errors::*;
