use alloc::vec::Vec;

use int_interval::UsizeCO;

use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn set(&mut self, index: usize, value: bool) -> Option<bool> {
        if index >= self.len {
            return None;
        }

        let old = Bits::bit_at(&self.bits, index);
        Bits::set_bit(&mut self.bits, index, value);
        Some(old)
    }

    /// Writes `len` bits of `value` starting at `bit_start`, OR-ing them
    /// with the existing bits.  Bits beyond `self.len()` are ignored.
    ///
    /// Only the low `len` bits of `value` are used; higher bits are
    /// masked out.
    #[inline]
    pub fn set_chunk(&mut self, bit_start: usize, value: u64, len: usize) {
        let value = value & Bits::low_mask(len);
        let word = bit_start / WORD_BITS;
        let shift = bit_start % WORD_BITS;

        if let Some(w) = self.bits.get_mut(word) {
            *w |= value << shift;
        }

        if shift != 0 {
            if let Some(w) = self.bits.get_mut(word + 1) {
                *w |= value >> (WORD_BITS - shift);
            }
        }
    }

    pub fn push(&mut self, value: bool) {
        let new_len = self.len.checked_add(1).expect("bit string length overflow");
        let new_words = Bits::word_len(new_len);

        if new_words != self.bits.len() {
            let mut bits = Vec::with_capacity(new_words);
            bits.extend_from_slice(&self.bits);
            bits.push(0);
            self.bits = bits.into_boxed_slice();
        }

        if value {
            Bits::set_bit(&mut self.bits, self.len, true);
        }

        self.len = new_len;
    }

    pub fn pop(&mut self) -> Option<bool> {
        let index = self.len.checked_sub(1)?;
        let value = Bits::bit_at(&self.bits, index);

        Bits::set_bit(&mut self.bits, index, false);
        self.len = index;

        let words = Bits::word_len(self.len);
        if words != self.bits.len() {
            self.bits = Bits::shrink_words(&self.bits, words);
        } else {
            Bits::mask_unused(&mut self.bits, self.len);
        }

        Some(value)
    }

    pub fn truncate(&mut self, len: usize) {
        assert!(
            len <= self.len,
            "cannot truncate bit string from len {} to larger len {}",
            self.len,
            len
        );

        if len == self.len {
            return;
        }

        self.len = len;

        let words = Bits::word_len(len);
        if words != self.bits.len() {
            self.bits = Bits::shrink_words(&self.bits, words);
        }

        Bits::mask_unused(&mut self.bits, len);
    }

    pub fn clear(&mut self) {
        self.bits = Bits::zero_words(0);
        self.len = 0;
    }

    pub fn insert(&mut self, index: usize, value: bool) {
        assert!(
            index <= self.len,
            "bit string insert index out of bounds: index={}, len={}",
            index,
            self.len
        );

        if index == self.len {
            self.push(value);
            return;
        }

        let new_len = self.len.checked_add(1).expect("bit string length overflow");
        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, index);
        Bits::set_bit(&mut bits, index, value);
        Bits::copy(&self.bits, index, &mut bits, index + 1, self.len - index);

        self.bits = bits;
        self.len = new_len;
    }

    pub fn remove(&mut self, index: usize) -> bool {
        assert!(
            index < self.len,
            "bit string remove index out of bounds: index={}, len={}",
            index,
            self.len
        );

        let value = Bits::bit_at(&self.bits, index);
        let new_len = self.len - 1;
        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, index);
        Bits::copy(
            &self.bits,
            index + 1,
            &mut bits,
            index,
            self.len - index - 1,
        );

        self.bits = bits;
        self.len = new_len;

        value
    }

    pub fn push_bits(&mut self, rhs: &Self) {
        if rhs.len == 0 {
            return;
        }

        if self.len == 0 {
            self.bits = rhs.bits.clone();
            self.len = rhs.len;
            return;
        }

        let old_len = self.len;
        let new_len = old_len
            .checked_add(rhs.len)
            .expect("bit string length overflow");

        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, self.len);
        Bits::copy(&rhs.bits, 0, &mut bits, old_len, rhs.len);

        self.bits = bits;
        self.len = new_len;
    }

    pub fn insert_bits(&mut self, index: usize, rhs: &Self) {
        assert!(
            index <= self.len,
            "bit string insert index out of bounds: index={}, len={}",
            index,
            self.len
        );

        if rhs.len == 0 {
            return;
        }

        if index == self.len {
            self.push_bits(rhs);
            return;
        }

        let new_len = self
            .len
            .checked_add(rhs.len)
            .expect("bit string length overflow");

        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, index);
        Bits::copy(&rhs.bits, 0, &mut bits, index, rhs.len);
        Bits::copy(
            &self.bits,
            index,
            &mut bits,
            index + rhs.len,
            self.len - index,
        );

        self.bits = bits;
        self.len = new_len;
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(
            at <= self.len,
            "bit string split index out of bounds: index={}, len={}",
            at,
            self.len
        );

        let rhs_len = self.len - at;
        let mut rhs_bits = Bits::zero_words(Bits::word_len(rhs_len));

        Bits::copy(&self.bits, at, &mut rhs_bits, 0, rhs_len);
        self.truncate(at);

        Self {
            bits: rhs_bits,
            len: rhs_len,
        }
    }

    pub fn replace_interval(&mut self, interval: UsizeCO, replacement: &Self) {
        Bits::assert_interval_in_bounds(interval, self.len);

        let start = interval.start();
        let end = interval.end_excl();
        let tail_len = self.len - end;

        let new_len = start
            .checked_add(replacement.len)
            .and_then(|len| len.checked_add(tail_len))
            .expect("bit string length overflow");

        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, start);
        Bits::copy(&replacement.bits, 0, &mut bits, start, replacement.len);
        Bits::copy(
            &self.bits,
            end,
            &mut bits,
            start + replacement.len,
            tail_len,
        );

        self.bits = bits;
        self.len = new_len;
    }

    pub fn drain_interval(&mut self, interval: UsizeCO) -> Self {
        Bits::assert_interval_in_bounds(interval, self.len);

        let start = interval.start();
        let end = interval.end_excl();
        let removed_len = interval.len();
        let tail_len = self.len - end;

        let mut removed_bits = Bits::zero_words(Bits::word_len(removed_len));
        Bits::copy(&self.bits, start, &mut removed_bits, 0, removed_len);

        let new_len = self.len - removed_len;
        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, start);
        Bits::copy(&self.bits, end, &mut bits, start, tail_len);

        self.bits = bits;
        self.len = new_len;

        Self {
            bits: removed_bits,
            len: removed_len,
        }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(bool) -> bool,
    {
        let mut write = 0usize;

        for read in 0..self.len {
            let value = Bits::bit_at(&self.bits, read);

            if f(value) {
                Bits::set_bit(&mut self.bits, write, value);
                write += 1;
            }
        }

        self.truncate(write);
    }
}

impl BitString {
    pub fn slice(&self, interval: UsizeCO) -> Self {
        Bits::assert_interval_in_bounds(interval, self.len);

        let start = interval.start();
        let len = interval.len();

        let mut bits = Bits::zero_words(Bits::word_len(len));
        Bits::copy(&self.bits, start, &mut bits, 0, len);

        Self { bits, len }
    }

    pub fn slice_from(&self, start: usize) -> Self {
        assert!(
            start <= self.len,
            "bit string slice start out of bounds: start={}, len={}",
            start,
            self.len
        );

        let len = self.len - start;

        if len == 0 {
            return Self::new();
        }

        let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
        self.slice(interval)
    }

    pub fn slice_until(&self, end: usize) -> Self {
        assert!(
            end <= self.len,
            "bit string slice end out of bounds: end={}, len={}",
            end,
            self.len
        );

        if end == 0 {
            return Self::new();
        }

        let interval = UsizeCO::checked_from_start_len(0, end).unwrap();
        self.slice(interval)
    }
}

impl Extend<bool> for BitString {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = bool>,
    {
        let rhs = Self::from_bool_iter(iter);
        self.push_bits(&rhs);
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
mod tests_for_set;
#[cfg(test)]
mod tests_for_set_chunk;

#[cfg(test)]
mod tests_for_push;

#[cfg(test)]
mod tests_for_pop;

#[cfg(test)]
mod tests_for_truncate;

#[cfg(test)]
mod tests_for_insert;

#[cfg(test)]
mod tests_for_remove;

#[cfg(test)]
mod tests_for_push_bits;

#[cfg(test)]
mod tests_for_insert_bits;

#[cfg(test)]
mod tests_for_split_off;

#[cfg(test)]
mod tests_for_replace_interval;

#[cfg(test)]
mod tests_for_drain_interval;

#[cfg(test)]
mod tests_for_retain;

#[cfg(test)]
mod tests_for_slice;

#[cfg(test)]
mod tests_for_extend;
