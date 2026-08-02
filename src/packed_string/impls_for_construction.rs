use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    const VALID_WIDTH: () = assert!(BITS <= 8, "packed character width must not exceed 8");

    #[inline]
    pub fn new() -> Self {
        let () = Self::VALID_WIDTH;
        Self {
            bits: BitString::new(),
            char_len: 0,
            marker: PhantomData,
        }
    }

    /// Packs enum values directly, without an alphabet lookup.
    pub fn from_chars<I>(chars: I) -> Self
    where
        I: IntoIterator<Item = C>,
    {
        let mut result = Self::new();
        result.extend(chars);
        result
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
