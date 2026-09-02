use core::iter::FusedIterator;

use super::*;

impl<'ps, C, const BITS: u8> PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn iter(&self) -> Iter<'ps, C, BITS> {
        Iter {
            view: *self,
            front: 0,
            back: self.char_len(),
        }
    }
}

#[allow(dead_code)]
pub struct Iter<'ps, C, const BITS: u8>
where
    C: PackedChar<BITS>,
{
    view: PackedStr<'ps, C, BITS>,
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
            None
        } else {
            let c = self.view.get(self.front);
            self.front += 1;
            c
        }
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
            None
        } else {
            self.back -= 1;
            self.view.get(self.back)
        }
    }
}

impl<C, const BITS: u8> ExactSizeIterator for Iter<'_, C, BITS> where C: PackedChar<BITS> {}
impl<C, const BITS: u8> FusedIterator for Iter<'_, C, BITS> where C: PackedChar<BITS> {}

impl<'ps, C, const BITS: u8> IntoIterator for &'ps PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    type Item = C;
    type IntoIter = Iter<'ps, C, BITS>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
