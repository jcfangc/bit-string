use super::*;

impl Extend<bool> for BitString {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = bool>,
    {
        let rhs = Self::from_bool_iter(iter);
        self.push_bit_string(&rhs);
    }
}

impl<'a> Extend<&'a bool> for BitString {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a bool>,
    {
        self.extend(iter.into_iter().copied());
    }
}

#[cfg(test)]
mod tests_for_extend;
