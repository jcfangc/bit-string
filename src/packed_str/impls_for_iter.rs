use core::iter::FusedIterator;

use super::*;

impl<'ps, C: PackedChar> PackedStr<'ps, C> {
    pub fn iter(&self) -> Iter<'ps, C> {
        unimplemented!("PackedStr::iter")
    }
}

#[allow(dead_code)]
pub struct Iter<'ps, C: PackedChar> {
    view: PackedStr<'ps, C>,
    front: usize,
    back: usize,
}

impl<C: PackedChar> Iterator for Iter<'_, C> {
    type Item = C;

    fn next(&mut self) -> Option<C> {
        unimplemented!("PackedStr::Iter::next")
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        unimplemented!("PackedStr::Iter::size_hint")
    }
}

impl<C: PackedChar> DoubleEndedIterator for Iter<'_, C> {
    fn next_back(&mut self) -> Option<C> {
        unimplemented!("PackedStr::Iter::next_back")
    }
}

impl<C: PackedChar> ExactSizeIterator for Iter<'_, C> {}
impl<C: PackedChar> FusedIterator for Iter<'_, C> {}

impl<'ps, C: PackedChar> IntoIterator for &'ps PackedStr<'ps, C> {
    type Item = C;
    type IntoIter = Iter<'ps, C>;

    fn into_iter(self) -> Self::IntoIter {
        unimplemented!("PackedStr::into_iter")
    }
}
