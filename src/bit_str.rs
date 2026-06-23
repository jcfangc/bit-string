use crate::BitString;

/// A zero-copy borrowed view of a [`BitString`] or subrange thereof.
///
/// `BitStr` is to `BitString` as `&str` is to `String` — it references the
/// underlying data without allocation, and carries a start offset plus length
/// for bit-level slicing.
///
/// # Size
///
/// 24 bytes on 64-bit targets (1 pointer + 2 usizes).
///
/// # Lifetime
///
/// The lifetime `'bs` is tied to the source [`BitString`].  While any `BitStr`
/// is live, Rust's borrow checker prevents mutation of the source.
#[derive(Clone, Copy)]
pub struct BitStr<'bs> {
    pub(crate) source: &'bs BitString,
    pub(crate) start: usize,
    /// Number of bits in this view.
    pub(crate) bit_len: usize,
}

pub mod errors;
mod impls_for_access;
mod impls_for_bit_arith;
mod impls_for_predicates;
mod impls_for_slice;

// ---------------------------------------------------------------------------
// Getters
// ---------------------------------------------------------------------------

impl<'bs> BitStr<'bs> {
    /// The bit offset of this view within its source [`BitString`].
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    /// The number of bits in this view.
    #[inline]
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    #[inline]
    pub fn source(&self) -> &BitString {
        self.source
    }
}
