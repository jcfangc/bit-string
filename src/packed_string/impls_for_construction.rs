use super::*;
use crate::packed_string::funcs_for_code::pack_codes;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    #[inline]
    pub fn new() -> Self {
        assert_valid_width::<BITS>();
        Self {
            bits: BitString::new(),
            marker: PhantomData,
        }
    }

    /// Packs enum values directly, without an alphabet lookup.
    pub fn from_chars<I>(chars: I) -> Self
    where
        I: IntoIterator<Item = C>,
    {
        Self::from_valid_bits(pack_codes::<C, BITS, I>(chars))
    }
}

impl<C, const BITS: u8> Default for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C, const BITS: u8> FromIterator<C> for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn from_iter<I: IntoIterator<Item = C>>(iter: I) -> Self {
        Self::from_chars(iter)
    }
}

#[cfg(test)]
mod tests_for_construction;

mod impls_for_from_bits;
mod impls_for_from_slice;
mod impls_for_repeat;
