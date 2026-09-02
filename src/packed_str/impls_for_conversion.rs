use super::*;

impl<'ps, C, const BITS: u8> PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn to_packed_string(&self) -> PackedString<C, BITS> {
        PackedString::from_bits(self.bits.to_bit_string()).expect("PackedStr invariant violated")
    }
}

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn as_packed_str(&self) -> PackedStr<'_, C, BITS> {
        PackedStr {
            bits: self.bits().as_bit_str(),
            marker: core::marker::PhantomData,
        }
    }
}
