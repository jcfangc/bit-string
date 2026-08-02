use super::*;

impl<C: PackedChar> PackedString<C> {
    /// Appends the character's code directly to the payload.
    pub fn push(&mut self, character: C) {
        let code = checked_code(character);
        let new_len = self
            .char_len
            .checked_add(1)
            .expect("packed string length overflow");
        for shift in 0..C::BITS {
            self.bits.push((code >> shift) & 1 != 0);
        }
        self.char_len = new_len;
    }

    pub fn pop(&mut self) -> Option<C> {
        let character = self.last()?;
        self.char_len -= 1;
        self.bits.truncate(self.char_len * usize::from(C::BITS));
        Some(character)
    }

    pub fn set(&mut self, index: usize, character: C) -> Option<C> {
        let previous = self.get(index)?;
        write_code(&mut self.bits, index, C::BITS, checked_code(character));
        Some(previous)
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.char_len {
            return;
        }
        self.char_len = new_len;
        self.bits.truncate(new_len * usize::from(C::BITS));
    }

    pub fn clear(&mut self) {
        self.char_len = 0;
        self.bits.clear();
    }
}

impl<C: PackedChar> Extend<C> for PackedString<C> {
    fn extend<I: IntoIterator<Item = C>>(&mut self, iter: I) {
        for character in iter {
            self.push(character);
        }
    }
}

impl<'a, C: PackedChar> Extend<&'a C> for PackedString<C> {
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
