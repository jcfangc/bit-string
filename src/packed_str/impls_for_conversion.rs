use super::*;

impl<'ps, C, const BITS: u8> PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn to_packed_string(&self) -> PackedString<C, BITS> {
        PackedString::from_valid_bits(self.bits.to_bit_string())
    }

    #[inline]
    pub(crate) fn from_valid_bit_str(bits: BitStr<'ps>) -> Self {
        let width = usize::from(BITS);
        crate::assert_valid_width::<BITS>();
        debug_assert_eq!(bits.start() % width, 0);
        debug_assert_eq!(bits.bit_len() % width, 0);
        Self {
            bits,
            marker: core::marker::PhantomData,
        }
    }
}

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn as_packed_str(&self) -> PackedStr<'_, C, BITS> {
        PackedStr::from_valid_bit_str(self.bits().as_bit_str())
    }
}
