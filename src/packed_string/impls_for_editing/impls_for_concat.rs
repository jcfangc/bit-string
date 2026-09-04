use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn push_packed_string(&mut self, other: &Self) {
        self.bits.push_bit_string(&other.bits);
    }

    pub fn insert_packed_string(&mut self, index: usize, other: &Self) {
        let start = index.min(self.char_len()) * usize::from(BITS);
        self.bits.insert_bit_string(start, &other.bits);
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        let at = at.min(self.char_len()) * usize::from(BITS);
        Self::from_bits(self.bits.split_off(at)).expect("PackedString invariant violated")
    }
}
