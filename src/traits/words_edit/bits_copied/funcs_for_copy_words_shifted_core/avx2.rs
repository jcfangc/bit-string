use crate::WORD_BITS;

#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm_set1_epi64x, _mm256_loadu_si256, _mm256_or_si256, _mm256_sll_epi64,
    _mm256_srl_epi64, _mm256_storeu_si256,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm_set1_epi64x, _mm256_loadu_si256, _mm256_or_si256, _mm256_sll_epi64,
    _mm256_srl_epi64, _mm256_storeu_si256,
};

#[target_feature(enable = "avx2")]
pub(super) unsafe fn copy_words_shifted(dst: &mut [u64], src: &[u64], len: usize, shift: usize) {
    // SAFETY: `#[target_feature(enable = "avx2")]` guarantees the CPU supports AVX2; the `unsafe fn` contract guarantees pointer validity.
    let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
    // SAFETY: Same as above.
    let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
    let mut i = 0;
    while i + 4 <= len {
        // SAFETY: `src` pointers are valid for `len + 1` words (caller guarantee); `_mm256_loadu_si256` uses unaligned loads so alignment is not required.
        let w0 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
        // SAFETY: Same as above.
        let w1 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i + 1).cast::<__m256i>()) };
        // SAFETY: Pure register operations; no memory access.
        let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
        // SAFETY: Same as above.
        let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
        // SAFETY: Same as above.
        let window = unsafe { _mm256_or_si256(lo, hi) };
        // SAFETY: `dst` pointer is valid for `len` words (caller guarantee); `_mm256_storeu_si256` uses unaligned stores.
        unsafe { _mm256_storeu_si256(dst.as_mut_ptr().add(i).cast::<__m256i>(), window) };
        i += 4;
    }
    while i < len {
        dst[i] = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
        i += 1;
    }
}
