use core::iter::FusedIterator;

use super::*;

impl<'ps, C, const BITS: u8> PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn iter(&self) -> Iter<'ps, C, BITS> {
        unimplemented!("PackedStr::iter")
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
        unimplemented!("PackedStr::Iter::next")
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        unimplemented!("PackedStr::Iter::size_hint")
    }
}

impl<C, const BITS: u8> DoubleEndedIterator for Iter<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn next_back(&mut self) -> Option<C> {
        unimplemented!("PackedStr::Iter::next_back")
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
        unimplemented!("PackedStr::into_iter")
    }
}
