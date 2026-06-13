use alloc::boxed::Box;
use alloc::vec::Vec;
use int_interval::UsizeCO;

use crate::WORD_BITS;

pub(crate) struct Bits;

impl Bits {
    #[inline]
    pub(crate) fn last_word_mask(len: usize) -> u64 {
        let rem = len % WORD_BITS;
        if rem == 0 {
            u64::MAX
        } else {
            (1u64 << rem) - 1
        }
    }

    #[inline]
    pub(crate) fn mask_unused(bits: &mut [u64], len: usize) {
        if let Some(last) = bits.last_mut() {
            *last &= Self::last_word_mask(len);
        }
    }

    #[inline]
    pub(crate) fn word_len(bit_len: usize) -> usize {
        bit_len / WORD_BITS + usize::from(bit_len % WORD_BITS != 0)
    }

    #[inline]
    pub(crate) fn zero_words(words: usize) -> Box<[u64]> {
        let mut bits = Vec::with_capacity(words);
        bits.resize(words, 0);
        bits.into_boxed_slice()
    }

    #[inline]
    pub(crate) fn shrink_words(bits: &[u64], words: usize) -> Box<[u64]> {
        let mut out = Vec::with_capacity(words);
        out.extend_from_slice(&bits[..words]);
        out.into_boxed_slice()
    }

    #[inline]
    pub(crate) fn bit_at(bits: &[u64], index: usize) -> bool {
        bits[index / WORD_BITS] & (1u64 << (index % WORD_BITS)) != 0
    }

    #[inline]
    pub(crate) fn set_bit(bits: &mut [u64], index: usize, value: bool) {
        let word = index / WORD_BITS;
        let mask = 1u64 << (index % WORD_BITS);

        if value {
            bits[word] |= mask;
        } else {
            bits[word] &= !mask;
        }
    }

    #[inline]
    pub(crate) fn low_mask(bits: usize) -> u64 {
        if bits == WORD_BITS {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        }
    }

    #[inline]
    pub(crate) fn read_chunk(src: &[u64], bit_start: usize) -> u64 {
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
    pub(crate) fn write_chunk(dst: &mut [u64], bit_start: usize, value: u64, len: usize) {
        let value = value & Self::low_mask(len);
        let word = bit_start / WORD_BITS;
        let shift = bit_start % WORD_BITS;

        dst[word] |= value << shift;

        if shift != 0 && word + 1 < dst.len() {
            dst[word + 1] |= value >> (WORD_BITS - shift);
        }
    }

    #[inline]
    pub(crate) fn copy(
        src: &[u64],
        src_start: usize,
        dst: &mut [u64],
        dst_start: usize,
        len: usize,
    ) {
        let mut done = 0usize;

        while done < len {
            let take = (len - done).min(WORD_BITS);
            let chunk = Self::read_chunk(src, src_start + done);
            Self::write_chunk(dst, dst_start + done, chunk, take);
            done += take;
        }
    }

    #[inline]
    pub(crate) fn assert_interval_in_bounds(interval: UsizeCO, len: usize) {
        assert!(
            interval.end_excl() <= len,
            "bit string interval out of bounds: {}..{}, len={}",
            interval.start(),
            interval.end_excl(),
            len
        );
    }
}

#[cfg(test)]
mod tests_for_assert_interval_in_bounds;
#[cfg(test)]
mod tests_for_bit_at;
#[cfg(test)]
mod tests_for_copy;
#[cfg(test)]
mod tests_for_last_word_mask;
#[cfg(test)]
mod tests_for_low_mask;
#[cfg(test)]
mod tests_for_mask_unused;
#[cfg(test)]
mod tests_for_read_chunk;
#[cfg(test)]
mod tests_for_set_bit;
#[cfg(test)]
mod tests_for_shrink_words;
#[cfg(test)]
mod tests_for_word_len;
#[cfg(test)]
mod tests_for_write_chunk;
#[cfg(test)]
mod tests_for_zero_words;
