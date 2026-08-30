use crate::WORD_BITS;
use core::arch::aarch64::{vdupq_n_s64, vld1q_u64, vorrq_u64, vshlq_u64, vst1q_u64};

#[target_feature(enable = "neon")]
pub(super) unsafe fn copy_words_shifted(dst: &mut [u64], src: &[u64], len: usize, shift: usize) {
    // SAFETY: `#[target_feature(enable = "neon")]` guarantees the CPU supports NEON; the `unsafe fn` contract guarantees pointer validity.
    let neg_shift = unsafe { vdupq_n_s64(-(shift as i64)) };
    // SAFETY: Same as above.
    let pos_shift = unsafe { vdupq_n_s64((WORD_BITS - shift) as i64) };

    let mut i = 0;
    while i + 2 <= len {
        // SAFETY: `src` pointers are valid for `len + 1` words (caller guarantee).
        let w0 = unsafe { vld1q_u64(src.as_ptr().add(i)) };
        // SAFETY: Same as above.
        let w1 = unsafe { vld1q_u64(src.as_ptr().add(i + 1)) };
        // SAFETY: Pure register operations; no memory access.
        let lo = unsafe { vshlq_u64(w0, neg_shift) };
        // SAFETY: Same as above.
        let hi = unsafe { vshlq_u64(w1, pos_shift) };
        // SAFETY: Same as above.
        let window = unsafe { vorrq_u64(lo, hi) };
        // SAFETY: `dst` pointer is valid for `len` words (caller guarantee).
        unsafe { vst1q_u64(dst.as_mut_ptr().add(i), window) };
        i += 2;
    }
    while i < len {
        dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
        i += 1;
    }
}
