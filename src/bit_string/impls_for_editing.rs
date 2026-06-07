use alloc::{boxed::Box, vec::Vec};

use int_interval::UsizeCO;

use crate::bit_string::funcs_for_share::mask_unused_bits;

use super::*;

#[inline]
fn word_len(bit_len: usize) -> usize {
    bit_len / WORD_BITS + usize::from(bit_len % WORD_BITS != 0)
}

#[inline]
fn zero_words(words: usize) -> Box<[u64]> {
    let mut bits = Vec::with_capacity(words);
    bits.resize(words, 0);
    bits.into_boxed_slice()
}

#[inline]
fn shrink_words(bits: &[u64], words: usize) -> Box<[u64]> {
    let mut out = Vec::with_capacity(words);
    out.extend_from_slice(&bits[..words]);
    out.into_boxed_slice()
}

#[inline]
fn bit_at(bits: &[u64], index: usize) -> bool {
    bits[index / WORD_BITS] & (1u64 << (index % WORD_BITS)) != 0
}

#[inline]
fn set_bit(bits: &mut [u64], index: usize, value: bool) {
    let word = index / WORD_BITS;
    let mask = 1u64 << (index % WORD_BITS);

    if value {
        bits[word] |= mask;
    } else {
        bits[word] &= !mask;
    }
}

#[inline]
fn low_mask(bits: usize) -> u64 {
    if bits == WORD_BITS {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

#[inline]
fn read_chunk(src: &[u64], bit_start: usize) -> u64 {
    let word = bit_start / WORD_BITS;
    let shift = bit_start % WORD_BITS;

    let lo = src.get(word).copied().unwrap_or(0) >> shift;

    if shift == 0 {
        lo
    } else {
        let hi = src.get(word + 1).copied().unwrap_or(0);
        lo | (hi << (WORD_BITS - shift))
    }
}

#[inline]
fn write_chunk(dst: &mut [u64], bit_start: usize, value: u64, len: usize) {
    let value = value & low_mask(len);
    let word = bit_start / WORD_BITS;
    let shift = bit_start % WORD_BITS;

    dst[word] |= value << shift;

    if shift != 0 && word + 1 < dst.len() {
        dst[word + 1] |= value >> (WORD_BITS - shift);
    }
}

fn copy_bits(src: &[u64], src_start: usize, dst: &mut [u64], dst_start: usize, len: usize) {
    let mut done = 0usize;

    while done < len {
        let take = (len - done).min(WORD_BITS);
        let chunk = read_chunk(src, src_start + done);
        write_chunk(dst, dst_start + done, chunk, take);
        done += take;
    }
}

#[inline]
fn assert_interval_in_bounds(interval: UsizeCO, len: usize) {
    assert!(
        interval.end_excl() <= len,
        "bit string interval out of bounds: {}..{}, len={}",
        interval.start(),
        interval.end_excl(),
        len
    );
}

impl BitString {
    pub fn set(&mut self, index: usize, value: bool) -> Option<bool> {
        if index >= self.len {
            return None;
        }

        let old = bit_at(&self.bits, index);
        set_bit(&mut self.bits, index, value);
        Some(old)
    }

    pub fn push(&mut self, value: bool) {
        let new_len = self.len.checked_add(1).expect("bit string length overflow");
        let new_words = word_len(new_len);

        if new_words != self.bits.len() {
            let mut bits = Vec::with_capacity(new_words);
            bits.extend_from_slice(&self.bits);
            bits.push(0);
            self.bits = bits.into_boxed_slice();
        }

        if value {
            set_bit(&mut self.bits, self.len, true);
        }

        self.len = new_len;
    }

    pub fn pop(&mut self) -> Option<bool> {
        let index = self.len.checked_sub(1)?;
        let value = bit_at(&self.bits, index);

        set_bit(&mut self.bits, index, false);
        self.len = index;

        let words = word_len(self.len);
        if words != self.bits.len() {
            self.bits = shrink_words(&self.bits, words);
        } else {
            mask_unused_bits(&mut self.bits, self.len);
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

        let words = word_len(len);
        if words != self.bits.len() {
            self.bits = shrink_words(&self.bits, words);
        }

        mask_unused_bits(&mut self.bits, len);
    }

    pub fn clear(&mut self) {
        self.bits = zero_words(0);
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
        let mut bits = zero_words(word_len(new_len));

        copy_bits(&self.bits, 0, &mut bits, 0, index);
        set_bit(&mut bits, index, value);
        copy_bits(&self.bits, index, &mut bits, index + 1, self.len - index);

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

        let value = bit_at(&self.bits, index);
        let new_len = self.len - 1;
        let mut bits = zero_words(word_len(new_len));

        copy_bits(&self.bits, 0, &mut bits, 0, index);
        copy_bits(
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

        let mut bits = zero_words(word_len(new_len));

        copy_bits(&self.bits, 0, &mut bits, 0, self.len);
        copy_bits(&rhs.bits, 0, &mut bits, old_len, rhs.len);

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

        let mut bits = zero_words(word_len(new_len));

        copy_bits(&self.bits, 0, &mut bits, 0, index);
        copy_bits(&rhs.bits, 0, &mut bits, index, rhs.len);
        copy_bits(
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
        let mut rhs_bits = zero_words(word_len(rhs_len));

        copy_bits(&self.bits, at, &mut rhs_bits, 0, rhs_len);
        self.truncate(at);

        Self {
            bits: rhs_bits,
            len: rhs_len,
        }
    }

    pub fn replace_interval(&mut self, interval: UsizeCO, replacement: &Self) {
        assert_interval_in_bounds(interval, self.len);

        let start = interval.start();
        let end = interval.end_excl();
        let tail_len = self.len - end;

        let new_len = start
            .checked_add(replacement.len)
            .and_then(|len| len.checked_add(tail_len))
            .expect("bit string length overflow");

        let mut bits = zero_words(word_len(new_len));

        copy_bits(&self.bits, 0, &mut bits, 0, start);
        copy_bits(&replacement.bits, 0, &mut bits, start, replacement.len);
        copy_bits(
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
        assert_interval_in_bounds(interval, self.len);

        let start = interval.start();
        let end = interval.end_excl();
        let removed_len = interval.len();
        let tail_len = self.len - end;

        let mut removed_bits = zero_words(word_len(removed_len));
        copy_bits(&self.bits, start, &mut removed_bits, 0, removed_len);

        let new_len = self.len - removed_len;
        let mut bits = zero_words(word_len(new_len));

        copy_bits(&self.bits, 0, &mut bits, 0, start);
        copy_bits(&self.bits, end, &mut bits, start, tail_len);

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
            let value = bit_at(&self.bits, read);

            if f(value) {
                set_bit(&mut self.bits, write, value);
                write += 1;
            }
        }

        self.truncate(write);
    }

    pub fn slice(&self, interval: UsizeCO) -> Self {
        assert_interval_in_bounds(interval, self.len);

        let start = interval.start();
        let len = interval.len();

        let mut bits = zero_words(word_len(len));
        copy_bits(&self.bits, start, &mut bits, 0, len);

        Self { bits, len }
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
