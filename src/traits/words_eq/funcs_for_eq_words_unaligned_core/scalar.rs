use crate::WORD_BITS;

pub(super) fn eq_words(src: &[u64], other: &[u64], count: usize, shift: usize) -> bool {
    for i in 0..count {
        let w0 = src[i];
        let w1 = src[i + 1];
        if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
            return false;
        }
    }
    true
}
