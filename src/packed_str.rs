//! A zero-copy borrowed view into a [`PackedString`](crate::PackedString).

use core::marker::PhantomData;

use crate::{BitStr, PackedString, traits::PackedChar};

/// A character-aligned borrowed view into a [`PackedString`].
///
/// The view may be unaligned to a `u64` word boundary, but never to a packed
/// character boundary. Its representation maintains these invariants:
///
/// - `bits.start()` is a multiple of `BITS`;
/// - `bits.bit_len()` is a multiple of `BITS`;
/// - the bit range lies within its source [`BitString`](crate::BitString).
#[derive(Clone, Copy)]
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
