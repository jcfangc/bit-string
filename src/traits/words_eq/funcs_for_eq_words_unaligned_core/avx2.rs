use crate::WORD_BITS;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_set1_epi64x, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd,
    _mm256_or_si256, _mm256_sll_epi64, _mm256_srl_epi64,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm_set1_epi64x, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_pd,
    _mm256_or_si256, _mm256_sll_epi64, _mm256_srl_epi64,
};

#[target_feature(enable = "avx2")]
pub(super) unsafe fn eq_words_unaligned(
    src: &[u64],
    other: &[u64],
    len: usize,
    shift: usize,
) -> bool {
    // SAFETY: `shift` and `WORD_BITS - shift` fit in i64; AVX2 is enabled by `#[target_feature]`.
    let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
    // SAFETY: same as above
    let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
    let mut i = 0;
    while i + 4 <= len {
        // SAFETY: `#[target_feature(enable = "avx2")]` ensures AVX2 is enabled.
        // Pointers `src` and `other` are valid for `len+1` elements (caller guarantees extra word for window shift).
        let w0 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
        // SAFETY: same as above; load from `src[i+1]`.
        let w1 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i + 1).cast::<__m256i>()) };
        // SAFETY: `_mm256_srl_epi64`, `_mm256_sll_epi64`, `_mm256_or_si256`, `_mm256_cmpeq_epi64`, and `_mm256_movemask_pd` are pure register operations; AVX2 is enabled by `#[target_feature]`.
        let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
        // SAFETY: same as above
        let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
        // SAFETY: same as above
        let window = unsafe { _mm256_or_si256(lo, hi) };
        // SAFETY: AVX2 is enabled; pointer `other` is valid for `len` elements.
        let b = unsafe { _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>()) };
        // SAFETY: `_mm256_cmpeq_epi64` is a pure register operation; AVX2 is enabled by `#[target_feature]`.
        let cmp = unsafe { _mm256_cmpeq_epi64(window, b) };
        // SAFETY: same as above
        if unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32 != 0b1111 {
            return false;
        }
        i += 4;
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
