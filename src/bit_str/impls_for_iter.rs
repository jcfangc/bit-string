use super::*;

impl<'bs> BitStr<'bs> {
    /// Returns a double-ended iterator over the bits of this view.
    #[inline]
    pub fn iter(&self) -> Iter<'bs> {
        Iter {
            bitstr: *self,
            front: 0,
            back: self.bit_len,
        }
    }

    /// Collects the bits into a `Vec<bool>`.
    #[inline]
    pub fn to_bool_vec(&self) -> alloc::vec::Vec<bool> {
        self.iter().collect()
    }
}

/// A double-ended, exact-size iterator over the bits of a [`BitStr`].
pub struct Iter<'bs> {
    bitstr: BitStr<'bs>,
    front: usize,
    back: usize,
}

impl Iterator for Iter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        // SAFETY: `front < back <= bit_len`, so `get(front)` is Some.
        let value = self.bitstr.get(self.front).unwrap();
        self.front += 1;
        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        // SAFETY: `back >= front` after decrement, so `get(back)` is Some.
        Some(self.bitstr.get(self.back).unwrap())
    }
}

impl ExactSizeIterator for Iter<'_> {}
impl core::iter::FusedIterator for Iter<'_> {}

impl<'bs> IntoIterator for &'bs BitStr<'bs> {
    type Item = bool;
    type IntoIter = Iter<'bs>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests_for_iter;
