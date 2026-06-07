use super::*;

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
pub(crate) fn mask_unused_bits(bits: &mut [u64], len: usize) {
    if let Some(last) = bits.last_mut() {
        *last &= last_word_mask(len);
    }
}
