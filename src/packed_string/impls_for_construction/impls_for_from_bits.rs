use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Validates and adopts an already packed bit payload.
    pub fn from_bits(_bits: BitString, _char_len: usize) -> Option<Self> {
        unimplemented!("PackedString::from_bits")
    }

    pub fn into_bits(self) -> BitString {
        unimplemented!("PackedString::into_bits")
    }

    pub fn into_parts(self) -> (BitString, usize) {
        unimplemented!("PackedString::into_parts")
    }
}
