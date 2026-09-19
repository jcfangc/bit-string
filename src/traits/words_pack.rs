use crate::{WORD_BITS, assert_valid_width};

/// Fixed-width code packing operations on `[u64]` backing storage.
///
/// The destination is expected to contain exactly the words produced by the
/// complete layout blocks in `codes`. Partial blocks and bit offsets are
/// handled by the packed-domain caller instead. Every code must fit in
/// `BITS` bits; callers are responsible for preserving this invariant. Every
/// destination word is fully overwritten, and its previous contents are
/// ignored.
pub(crate) trait WordsPack {
    fn pack_codes<const BITS: u8>(&mut self, codes: &[u8]);
}

/// Returns the smallest number of codes that occupies a whole number of words.
#[inline]
pub(crate) const fn layout_block_len<const BITS: u8>() -> usize {
    assert_valid_width::<BITS>();
    WORD_BITS / gcd(WORD_BITS, BITS as usize)
}

#[inline]
const fn gcd(mut lhs: usize, mut rhs: usize) -> usize {
    while rhs != 0 {
        let remainder = lhs % rhs;
        lhs = rhs;
        rhs = remainder;
    }
    lhs
}

pub(crate) mod funcs_for_pack_codes_core;
mod impls_for_u64_slice;
