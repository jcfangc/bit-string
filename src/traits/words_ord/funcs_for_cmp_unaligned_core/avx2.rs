use core::cmp::Ordering;

use crate::WORD_BITS;
use crate::traits::WordOrd;

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
pub(super) unsafe fn cmp_unaligned(
    src: &[u64],
    other: &[u64],
    len: usize,
    shift: usize,
) -> Option<Ordering> {
    let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
    let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
    let mut i = 0;
    while i + 4 <= len {
        // Load src[i..i+4] and src[i+1..i+5].
        let w0 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i).cast::<__m256i>()) };
        let w1 = unsafe { _mm256_loadu_si256(src.as_ptr().add(i + 1).cast::<__m256i>()) };
        let lo = unsafe { _mm256_srl_epi64(w0, count_lo) };
        let hi = unsafe { _mm256_sll_epi64(w1, count_hi) };
        let window = unsafe { _mm256_or_si256(lo, hi) };
        let b = unsafe { _mm256_loadu_si256(other.as_ptr().add(i).cast::<__m256i>()) };
        let cmp = unsafe { _mm256_cmpeq_epi64(window, b) };
        let mask = unsafe { _mm256_movemask_pd(core::mem::transmute(cmp)) } as u32;
        if mask != 0b1111 {
            let lane = mask.trailing_ones() as usize;
            let sw = (src[i + lane] >> shift) | (src[i + lane + 1] << (WORD_BITS - shift));
            return Some(WordOrd::bitwise_cmp(sw, other[i + lane]));
        }
        i += 4;
    }
    while i < len {
        let sw = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
        if sw != other[i] {
            return Some(WordOrd::bitwise_cmp(sw, other[i]));
        }
        i += 1;
    }
    None
}
