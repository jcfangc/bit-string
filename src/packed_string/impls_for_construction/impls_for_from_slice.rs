use super::*;

impl<C, const BITS: u8> From<&[C]> for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn from(characters: &[C]) -> Self {
        Self::from_chars(characters.iter().copied())
    }
}

impl<C, const BITS: u8, const N: usize> From<[C; N]> for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn from(characters: [C; N]) -> Self {
        Self::from_chars(characters)
    }
}
