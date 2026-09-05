use crate::WORD_BITS;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi64x,
    _mm_sll_epi64, _mm_srl_epi64,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi64x,
    _mm_sll_epi64, _mm_srl_epi64,
};

#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn eq_words_unaligned(
    src: &[u64],
    other: &[u64],
    len: usize,
    shift: usize,
) -> bool {
    // SAFETY: `shift` and `WORD_BITS - shift` fit in i64; SSE4.1 is enabled by `#[target_feature]`.
    let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
    // SAFETY: same as above
    let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
    let mut i = 0;
    while i + 2 <= len {
        // SAFETY: `#[target_feature(enable = "sse4.1")]` ensures SSE4.1 is enabled.
        // Pointers `src` and `other` are valid for `len+1` and `len` elements respectively (caller guarantees).
        let w0 = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
        // SAFETY: same as above; load from `src[i+1]`.
        let w1 = unsafe { _mm_loadu_si128(src.as_ptr().add(i + 1).cast::<__m128i>()) };
        // SAFETY: `_mm_srl_epi64`, `_mm_sll_epi64`, `_mm_or_si128`, `_mm_cmpeq_epi64`, and `_mm_movemask_epi8` are pure register operations; SSE4.1 is enabled by `#[target_feature]`.
        let lo = unsafe { _mm_srl_epi64(w0, count_lo) };
        // SAFETY: same as above
        let hi = unsafe { _mm_sll_epi64(w1, count_hi) };
        // SAFETY: same as above
        let window = unsafe { _mm_or_si128(lo, hi) };
        // SAFETY: SSE4.1 is enabled; pointer `other` is valid for `len` elements.
        let b = unsafe { _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>()) };
        // SAFETY: `_mm_cmpeq_epi64` is a pure register operation; SSE4.1 is enabled by `#[target_feature]`.
        let cmp = unsafe { _mm_cmpeq_epi64(window, b) };
        // SAFETY: same as above
        if unsafe { _mm_movemask_epi8(cmp) } as u32 != 0xFFFF {
            return false;
        }
        i += 2;
    }
    while i < len {
        let w0 = src[i];
        let w1 = src[i + 1];
        if ((w0 >> shift) | (w1 << (WORD_BITS - shift))) != other[i] {
            return false;
        }
        i += 1;
    }
    true
}
