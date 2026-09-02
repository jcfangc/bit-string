//! A zero-copy borrowed view into a [`PackedString`](crate::PackedString).

use core::marker::PhantomData;

use crate::{BitStr, PackedString, traits::PackedChar};

/// A character-aligned borrowed view into a [`PackedString`].
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PackedStr<'ps, C, const BITS: u8>
where
    C: PackedChar<BITS>,
{
    bits: BitStr<'ps>,
    marker: PhantomData<fn() -> C>,
}

mod impls_for_access;
mod impls_for_conversion;
mod impls_for_eq;
mod impls_for_iter;
mod impls_for_matching;
mod impls_for_ord;
mod impls_for_slice;
