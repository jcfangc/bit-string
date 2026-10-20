use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn insert(&mut self, index: usize, character: C) {
        let index = index.min(self.char_len());
        let start = index * usize::from(BITS);
        self.bits
            .insert_bits(start, u64::from(character.code()), usize::from(BITS));
    }

    pub fn remove(&mut self, index: usize) -> C {
        let character = self.get(index).expect("packed string index out of bounds");
        let start = index * usize::from(BITS);
        self.bits.remove_bits(start, usize::from(BITS));
        character
    }
}

#[cfg(test)]
mod tests_for_insert_remove;
