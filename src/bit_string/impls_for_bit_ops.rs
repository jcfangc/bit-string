mod funcs_for_binary_core;
mod impls_for_count_ones;
mod impls_for_not;
mod impls_for_shl;

use alloc::vec::Vec;

use crate::bit_string::{errors::BitStringLenMismatch, funcs_for_share::mask_unused_bits};

use super::*;

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
mod tests_for_shr_zeros;
