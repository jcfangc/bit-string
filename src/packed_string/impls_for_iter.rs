use core::iter::FusedIterator;

use super::*;

impl<C: PackedChar> PackedString<C> {
    #[inline]
    pub fn iter(&self) -> Iter<'_, C> {
        Iter {
            string: self,
            front: 0,
            back: self.char_len,
        }
    }

    /// Collects the decoded characters into a vector.
    pub fn to_vec(&self) -> alloc::vec::Vec<C> {
        unimplemented!("PackedString::to_vec")
    }
}

#[derive(Clone)]
pub struct Iter<'a, C: PackedChar> {
    string: &'a PackedString<C>,
    front: usize,
    back: usize,
}

impl<C: PackedChar> Iterator for Iter<'_, C> {
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

impl<C: PackedChar> DoubleEndedIterator for Iter<'_, C> {
    fn next_back(&mut self) -> Option<C> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        self.string.get(self.back)
    }
}

impl<C: PackedChar> ExactSizeIterator for Iter<'_, C> {}
impl<C: PackedChar> FusedIterator for Iter<'_, C> {}

impl<'a, C: PackedChar> IntoIterator for &'a PackedString<C> {
    type Item = C;
    type IntoIter = Iter<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests_for_iter;
