use super::*;

impl<C, const BITS: u8> From<&[C]> for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn from(_characters: &[C]) -> Self {
        unimplemented!("PackedString::from(&[C])")
    }
}

impl<C, const BITS: u8, const N: usize> From<[C; N]> for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn from(_characters: [C; N]) -> Self {
        unimplemented!("PackedString::from([C; N])")
    }
}
