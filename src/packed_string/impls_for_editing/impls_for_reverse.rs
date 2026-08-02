use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn reverse(&self) -> Self {
        unimplemented!("PackedString::reverse")
    }

    pub fn reverse_assign(&mut self) {
        unimplemented!("PackedString::reverse_assign")
    }
}
