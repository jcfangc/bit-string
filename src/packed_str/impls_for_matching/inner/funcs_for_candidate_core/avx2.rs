#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_epi8,
    _mm256_set1_epi64x,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_loadu_si256, _mm256_movemask_epi8,
    _mm256_set1_epi64x,
};

#[target_feature(enable = "avx2")]
pub(super) unsafe fn matching_lanes(
    windows: &[u64; 4],
    needle_first_word: u64,
    needle_first_word_mask: u64,
) -> u8 {
    let candidates = unsafe { _mm256_loadu_si256(windows.as_ptr().cast::<__m256i>()) };
    let needle = _mm256_set1_epi64x(needle_first_word as i64);
    let mask = _mm256_set1_epi64x(needle_first_word_mask as i64);
    let candidates = _mm256_and_si256(candidates, mask);
    let equal = _mm256_cmpeq_epi64(candidates, needle);
    let byte_lanes = _mm256_movemask_epi8(equal) as u32;
    ((byte_lanes & 0x0000_00ff != 0) as u8)
        | (((byte_lanes & 0x0000_ff00 != 0) as u8) << 1)
        | (((byte_lanes & 0x00ff_0000 != 0) as u8) << 2)
        | (((byte_lanes & 0xff00_0000 != 0) as u8) << 3)
}
