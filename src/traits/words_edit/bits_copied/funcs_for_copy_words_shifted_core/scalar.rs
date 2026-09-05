use crate::WORD_BITS;

/// Scalar backend for shifted 64-bit windows.
#[inline]
pub(super) fn copy_words_shifted(dst: &mut [u64], src: &[u64], count: usize, shift: usize) {
    for i in 0..count {
        dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
    }
}
