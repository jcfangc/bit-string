use super::*;

impl BitString {
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            bit_string: self,
            front: 0,
            back: self.bit_len,
        }
    }
}

pub struct Iter<'a> {
    bit_string: &'a BitString,
    front: usize,
    back: usize,
}

impl Iterator for Iter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }

        let value = self.bit_string.get(self.front).unwrap();
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
        Some(self.bit_string.get(self.back).unwrap())
    }
}

impl ExactSizeIterator for Iter<'_> {}
impl core::iter::FusedIterator for Iter<'_> {}

impl<'a> IntoIterator for &'a BitString {
    type Item = bool;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests_for_iter;
