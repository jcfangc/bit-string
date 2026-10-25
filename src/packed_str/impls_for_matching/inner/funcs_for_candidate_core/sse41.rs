#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi64x,
};

#[target_feature(enable = "sse4.1")]
pub(super) unsafe fn matching_lanes(
    windows: &[u64; 4],
    needle_first_word: u64,
    needle_first_word_mask: u64,
) -> u8 {
    let needle = _mm_set1_epi64x(needle_first_word as i64);
    let mask = _mm_set1_epi64x(needle_first_word_mask as i64);
    let lower = unsafe { _mm_loadu_si128(windows.as_ptr().cast::<__m128i>()) };
    let upper = unsafe { _mm_loadu_si128(windows.as_ptr().add(2).cast::<__m128i>()) };
    let lower = _mm_cmpeq_epi64(_mm_and_si128(lower, mask), needle);
    let upper = _mm_cmpeq_epi64(_mm_and_si128(upper, mask), needle);
    let lower_bytes = _mm_movemask_epi8(lower) as u16;
    let upper_bytes = _mm_movemask_epi8(upper) as u16;
    let lower_lanes =
        ((lower_bytes & 0x00ff != 0) as u8) | (((lower_bytes & 0xff00 != 0) as u8) << 1);
    let upper_lanes =
        ((upper_bytes & 0x00ff != 0) as u8) | (((upper_bytes & 0xff00 != 0) as u8) << 1);
    lower_lanes | (upper_lanes << 2)
}
