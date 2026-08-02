use super::*;

impl<C: PackedChar> PartialEq for PackedString<C> {
    fn eq(&self, other: &Self) -> bool {
        self.char_len == other.char_len && self.bits == other.bits
    }
}

impl<C: PackedChar> Eq for PackedString<C> {}

#[cfg(test)]
mod tests_for_eq;
