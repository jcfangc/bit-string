use super::*;

impl<C: PackedChar> PackedString<C> {
    #[inline]
    pub fn new() -> Self {
        assert_valid_width::<C>();
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

impl<C: PackedChar> Default for PackedString<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: PackedChar> FromIterator<C> for PackedString<C> {
    fn from_iter<I: IntoIterator<Item = C>>(iter: I) -> Self {
        Self::from_chars(iter)
    }
}

#[cfg(test)]
mod tests_for_construction;

mod impls_for_from_bits;
mod impls_for_from_slice;
mod impls_for_repeat;
