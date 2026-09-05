use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Appends the character's code directly to the payload.
    pub fn push(&mut self, character: C) {
        let code = character.code();
        self.bits
            .bit_len()
            .checked_add(usize::from(BITS))
            .expect("packed string length overflow");
        for shift in 0..BITS {
            self.bits.push((code >> shift) & 1 != 0);
        }
    }

    pub fn pop(&mut self) -> Option<C> {
        let character = self.last()?;
        self.bits
            .truncate((self.char_len() - 1) * usize::from(BITS));
        Some(character)
    }

    pub fn set(&mut self, index: usize, character: C) -> Option<C> {
        let previous = self.get(index)?;
        write_code::<BITS>(&mut self.bits, index, character.code());
        Some(previous)
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.char_len() {
            return;
        }
        self.bits.truncate(new_len * usize::from(BITS));
    }

    pub fn clear(&mut self) {
        self.bits.clear();
    }
}

impl<C, const BITS: u8> Extend<C> for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn extend<I: IntoIterator<Item = C>>(&mut self, iter: I) {
        for character in iter {
            self.push(character);
        }
    }
}

impl<'a, C, const BITS: u8> Extend<&'a C> for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn extend<I: IntoIterator<Item = &'a C>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied());
    }
}

#[cfg(test)]
mod tests_for_editing;

mod impls_for_concat;
mod impls_for_drain;
mod impls_for_insert_remove;
mod impls_for_replace;
mod impls_for_retain;
mod impls_for_reverse;
mod impls_for_slice;
