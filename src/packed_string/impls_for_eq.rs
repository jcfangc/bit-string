use super::*;

impl<C, const BITS: u8> PartialEq for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl<C, const BITS: u8> Eq for PackedString<C, BITS> where C: PackedChar<BITS> {}

#[cfg(test)]
mod tests_for_eq;
