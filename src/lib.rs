#![no_std]

extern crate alloc;

mod consts_for_bits;
mod funcs_for_bits;
mod funcs_for_packed;

pub mod traits;
pub(crate) use consts_for_bits::*;
pub(crate) use funcs_for_bits::*;
pub(crate) use funcs_for_packed::*;

mod bit_str;
mod bit_string;
mod packed_str;
mod packed_string;

pub use bit_str::BitStr;
pub use bit_string::BitString;
pub use bit_string_derive::packed;
pub use packed_str::PackedStr;
pub use packed_string::PackedString;

pub use bit_str::errors::*;
pub use bit_string::errors::*;
