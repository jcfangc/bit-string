#![no_std]

extern crate alloc;

mod consts_for_bits;
mod funcs_for_bits;

pub(crate) use consts_for_bits::*;
pub(crate) use funcs_for_bits::*;

mod bit_string;

pub use bit_string::BitString;

pub use bit_string::errors::*;
