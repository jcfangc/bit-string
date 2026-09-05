use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn repeat(character: C, char_len: usize) -> Self {
        let mut result = Self::new();
        result.extend(core::iter::repeat_n(character, char_len));
        result
    }
}
