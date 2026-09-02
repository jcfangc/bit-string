//! A packed sequence whose character type defines its own bit encoding.

use core::marker::PhantomData;

use crate::{BitString, traits::PackedChar};

/// An owned sequence of [`PackedChar`] values stored in exactly `BITS` bits
/// per character.
///
/// The character type is the alphabet and its enum discriminant is the stored
/// value, so no alphabet allocation or lookup table is required.
#[derive(Clone)]
pub struct PackedString<C, const BITS: u8>
where
    C: PackedChar<BITS>,
{
    bits: BitString,
    marker: PhantomData<fn() -> C>,
}

pub(crate) mod funcs_for_code;
mod impls_for_access;
mod impls_for_construction;
mod impls_for_editing;
mod impls_for_eq;
mod impls_for_fmt;
mod impls_for_hash;
mod impls_for_iter;
mod impls_for_matching;
mod impls_for_ord;

use funcs_for_code::{code_mask, write_code};

#[cfg(test)]
mod tests_for_support;
