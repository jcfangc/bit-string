use alloc::vec::Vec;

use crate::bit_string::{
    errors::BitStringLenMismatch,
    funcs_for_share::{last_word_mask, mask_unused_bits},
};

use super::*;

mod and;

impl BitString {
    #[inline]
    fn require_same_len(&self, rhs: &Self) -> Result<(), BitStringLenMismatch> {
        if self.len == rhs.len {
            Ok(())
        } else {
            Err(BitStringLenMismatch {
                lhs_len: self.len,
                rhs_len: rhs.len,
            })
        }
    }

    fn binary_bits(
        &self,
        rhs: &Self,
        f: impl Fn(u64, u64) -> u64,
    ) -> Result<Self, BitStringLenMismatch> {
        self.require_same_len(rhs)?;

        let mut bits = Vec::with_capacity(self.bits.len());
        for (&lhs, &rhs) in self.bits.iter().zip(rhs.bits.iter()) {
            bits.push(f(lhs, rhs));
        }

        let mut bits = bits.into_boxed_slice();
        mask_unused_bits(&mut bits, self.len);

        Ok(Self {
            bits,
            len: self.len,
        })
    }

    #[inline]
    pub fn or_bits(&self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.binary_bits(rhs, |lhs, rhs| lhs | rhs)
    }

    #[inline]
    pub fn xor_bits(&self, rhs: &Self) -> Result<Self, BitStringLenMismatch> {
        self.binary_bits(rhs, |lhs, rhs| lhs ^ rhs)
    }

    #[inline]
    pub fn not_bits(&self) -> Self {
        let mut bits = self.bits.clone();

        for word in bits.iter_mut() {
            *word = !*word;
        }

        mask_unused_bits(&mut bits, self.len);

        Self {
            bits,
            len: self.len,
        }
    }

    #[inline]
    pub fn count_ones(&self) -> usize {
        let full_words = self.len / WORD_BITS;
        let rem = self.len % WORD_BITS;

        let mut count = self.bits[..full_words]
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum::<usize>();

        if rem != 0 {
            count += (self.bits[full_words] & last_word_mask(self.len)).count_ones() as usize;
        }

        count
    }

    #[inline]
    pub fn count_zeros(&self) -> usize {
        self.len - self.count_ones()
    }

    pub fn shl_zeros(&self, amount: usize) -> Self {
        let word_count = self.bits.len();
        let mut bits = Vec::new();
        bits.resize(word_count, 0u64);

        if amount >= self.len || self.len == 0 {
            return Self {
                bits: bits.into_boxed_slice(),
                len: self.len,
            };
        }

        let word_shift = amount / WORD_BITS;
        let bit_shift = amount % WORD_BITS;

        for (src_index, &word) in self.bits.iter().enumerate() {
            if word == 0 {
                continue;
            }

            let dst_index = src_index + word_shift;
            if dst_index >= word_count {
                continue;
            }

            bits[dst_index] |= word << bit_shift;

            if bit_shift != 0 && dst_index + 1 < word_count {
                bits[dst_index + 1] |= word >> (WORD_BITS - bit_shift);
            }
        }

        let mut bits = bits.into_boxed_slice();
        mask_unused_bits(&mut bits, self.len);

        Self {
            bits,
            len: self.len,
        }
    }

    pub fn shr_zeros(&self, amount: usize) -> Self {
        let word_count = self.bits.len();
        let mut bits = Vec::new();
        bits.resize(word_count, 0u64);

        if amount >= self.len || self.len == 0 {
            return Self {
                bits: bits.into_boxed_slice(),
                len: self.len,
            };
        }

        let word_shift = amount / WORD_BITS;
        let bit_shift = amount % WORD_BITS;

        for (src_index, &word) in self.bits.iter().enumerate().skip(word_shift) {
            if word == 0 {
                continue;
            }

            let dst_index = src_index - word_shift;

            bits[dst_index] |= word >> bit_shift;

            if bit_shift != 0 && dst_index > 0 {
                bits[dst_index - 1] |= word << (WORD_BITS - bit_shift);
            }
        }

        let mut bits = bits.into_boxed_slice();
        mask_unused_bits(&mut bits, self.len);

        Self {
            bits,
            len: self.len,
        }
    }
}

#[cfg(test)]
mod tests_for_or_bits;

#[cfg(test)]
mod tests_for_xor_bits;

#[cfg(test)]
mod tests_for_not_bits;

#[cfg(test)]
mod tests_for_count_ones;

#[cfg(test)]
mod tests_for_shl_zeros;

#[cfg(test)]
mod tests_for_shr_zeros;
