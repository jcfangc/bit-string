use core::cmp::Ordering;

use crate::WORD_BITS;
use crate::traits::WordOrd;

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
pub(super) unsafe fn cmp_unaligned(
    src: &[u64],
    other: &[u64],
    len: usize,
    shift: usize,
) -> Option<Ordering> {
    let count_lo = unsafe { _mm_set1_epi64x(shift as i64) };
    let count_hi = unsafe { _mm_set1_epi64x((WORD_BITS - shift) as i64) };
    let mut i = 0;
    while i + 2 <= len {
        // Load src[i..i+2] and src[i+1..i+3].
        let w0 = unsafe { _mm_loadu_si128(src.as_ptr().add(i).cast::<__m128i>()) };
        let w1 = unsafe { _mm_loadu_si128(src.as_ptr().add(i + 1).cast::<__m128i>()) };
        let lo = unsafe { _mm_srl_epi64(w0, count_lo) };
        let hi = unsafe { _mm_sll_epi64(w1, count_hi) };
        let window = unsafe { _mm_or_si128(lo, hi) };
        let b = unsafe { _mm_loadu_si128(other.as_ptr().add(i).cast::<__m128i>()) };
        let cmp = unsafe { _mm_cmpeq_epi64(window, b) };
        let mask = unsafe { _mm_movemask_epi8(cmp) } as u32;
        if mask != 0xFFFF {
            let lane = mask.trailing_ones() as usize / 8;
            let sw = (src[i + lane] >> shift) | (src[i + lane + 1] << (WORD_BITS - shift));
            return Some(WordOrd::bitwise_cmp(sw, other[i + lane]));
        }
        i += 2;
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
