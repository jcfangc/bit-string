use core::iter::FusedIterator;

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    #[inline]
    pub fn iter(&self) -> Iter<'_, C, BITS> {
        Iter {
            string: self,
            front: 0,
            back: self.char_len(),
        }
    }

    /// Collects the decoded characters into a vector.
    pub fn to_vec(&self) -> alloc::vec::Vec<C> {
        self.iter().collect()
    }
}

#[derive(Clone)]
pub struct Iter<'a, C, const BITS: u8>
where
    C: PackedChar<BITS>,
{
    string: &'a PackedString<C, BITS>,
    front: usize,
    back: usize,
}

impl<C, const BITS: u8> Iterator for Iter<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    type Item = C;

    fn next(&mut self) -> Option<C> {
        if self.front == self.back {
            return None;
        }
        let character = self.string.get(self.front);
        self.front += 1;
        character
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl<C, const BITS: u8> DoubleEndedIterator for Iter<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn next_back(&mut self) -> Option<C> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        self.string.get(self.back)
    }
}

impl<C, const BITS: u8> ExactSizeIterator for Iter<'_, C, BITS> where C: PackedChar<BITS> {}
impl<C, const BITS: u8> FusedIterator for Iter<'_, C, BITS> where C: PackedChar<BITS> {}

impl<'a, C, const BITS: u8> IntoIterator for &'a PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    type Item = C;
    type IntoIter = Iter<'a, C, BITS>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests_for_iter;
