use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn insert(&mut self, index: usize, character: C) {
        let index = index.min(self.char_len());
        self.bits
            .bit_len()
            .checked_add(usize::from(BITS))
            .expect("packed string length overflow");
        let start = index * usize::from(BITS);
        for shift in (0..BITS).rev() {
            self.bits
                .insert(start, (character.code() >> shift) & 1 != 0);
        }
    }

    pub fn remove(&mut self, index: usize) -> C {
        let character = self.get(index).expect("packed string index out of bounds");
        let start = index * usize::from(BITS);
        let mut code = 0;
        for shift in 0..BITS {
            if self.bits.remove(start) {
                code |= 1 << shift;
            }
        }
        debug_assert_eq!(code, character.code());
        character
    }
}
