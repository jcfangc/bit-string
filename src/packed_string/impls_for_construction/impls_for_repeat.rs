use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn repeat(_character: C, _char_len: usize) -> Self {
        unimplemented!("PackedString::repeat")
    }
}
