//! A packed sequence whose character type defines its own bit encoding.

use core::marker::PhantomData;

use crate::{BitString, traits::PackedChar};

/// An owned sequence of fixed-width [`PackedChar`] values.
///
/// The character type is the alphabet and its enum discriminant is the stored
/// value, so no alphabet allocation or lookup table is required.
#[derive(Clone)]
pub struct PackedString<C: PackedChar> {
    bits: BitString,
    char_len: usize,
    marker: PhantomData<fn() -> C>,
}

mod funcs_for_code;
mod impls_for_access;
mod impls_for_construction;
mod impls_for_editing;
mod impls_for_eq;
mod impls_for_fmt;
mod impls_for_iter;

use funcs_for_code::{assert_valid_width, checked_code, code_mask, write_code};

#[cfg(test)]
mod tests_for_support;
