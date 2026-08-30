use crate::WORD_BITS;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, _mm_loadu_si128, _mm_or_si128, _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
    _mm_storeu_si128,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_loadu_si128, _mm_or_si128, _mm_set1_epi64x, _mm_sll_epi64, _mm_srl_epi64,
    _mm_storeu_si128,
};

#[target_feature(enable = "sse2")]
pub(super) unsafe fn copy_words_shifted(dst: &mut [u64], src: &[u64], len: usize, shift: usize) {
    // SAFETY: `#[target_feature(enable = "sse2")]` guarantees the CPU supports SSE2; the `unsafe fn` contract guarantees pointer validity.
    let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
    // SAFETY: Same as above.
    let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
    let mut i = 0;
    while i + 2 <= len {
        // SAFETY: `src` pointers are valid for `len + 1` words (caller guarantee); `_mm_loadu_si128` uses unaligned loads so alignment is not required.
        let w0 = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
        // SAFETY: Same as above.
        let w1 = unsafe { _mm_loadu_si128(src.as_ptr().add(i + 1).cast::<__m128i>()) };
        // SAFETY: Pure register operations; no memory access.
        let lo = unsafe { _mm_srl_epi64(w0, count_lo) };
        // SAFETY: Same as above.
        let hi = unsafe { _mm_sll_epi64(w1, count_hi) };
        // SAFETY: Same as above.
        let window = unsafe { _mm_or_si128(lo, hi) };
        // SAFETY: `dst` pointer is valid for `len` words (caller guarantee); `_mm_storeu_si128` uses unaligned stores.
        unsafe { _mm_storeu_si128(dst.as_mut_ptr().add(i).cast::<__m128i>(), window) };
        i += 2;
    }
    while i < len {
        dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
        i += 1;
    }
}
