#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_i64gather_epi64, _mm256_loadu_si256,
    _mm256_movemask_epi8, _mm256_movemask_pd, _mm256_or_si256, _mm256_set1_epi64x,
    _mm256_setr_epi64x, _mm256_sllv_epi64, _mm256_srlv_epi64,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m256i, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_i64gather_epi64, _mm256_loadu_si256,
    _mm256_movemask_epi8, _mm256_movemask_pd, _mm256_or_si256, _mm256_set1_epi64x,
    _mm256_setr_epi64x, _mm256_sllv_epi64, _mm256_srlv_epi64,
};

use crate::{BitStr, WORD_BITS};

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

#[target_feature(enable = "avx2")]
pub(super) unsafe fn matching_candidates<const BITS: u8>(
    haystack: BitStr<'_>,
    first_char: usize,
    needle_first_word: u64,
    needle_first_word_mask: u64,
) -> Option<u8> {
    let width = usize::from(BITS);
    let first_bit = haystack.start() + first_char * width;
    let base_word = first_bit / WORD_BITS;
    let first_shift = first_bit % WORD_BITS;
    let source_words = haystack.source().words();
    let last_source_offset = (first_shift + 3 * width) / WORD_BITS + 1;
    if base_word
        .checked_add(last_source_offset)
        .is_none_or(|last_word| last_word >= source_words.len())
    {
        return None;
    }

    let mut low_word_offsets = [0i64; 4];
    let mut high_word_offsets = [0i64; 4];
    let mut shifts = [0i64; 4];
    let mut inverse_shifts = [0i64; 4];
    for lane in 0..4 {
        let lane_bit = first_shift + lane * width;
        let word_offset = lane_bit / WORD_BITS;
        let shift = lane_bit % WORD_BITS;
        low_word_offsets[lane] = word_offset as i64;
        high_word_offsets[lane] = (word_offset + 1) as i64;
        shifts[lane] = shift as i64;
        inverse_shifts[lane] = (WORD_BITS - shift) as i64;
    }

    let base = unsafe { source_words.as_ptr().add(base_word).cast::<i64>() };
    let low_indices = _mm256_setr_epi64x(
        low_word_offsets[0],
        low_word_offsets[1],
        low_word_offsets[2],
        low_word_offsets[3],
    );
    let high_indices = _mm256_setr_epi64x(
        high_word_offsets[0],
        high_word_offsets[1],
        high_word_offsets[2],
        high_word_offsets[3],
    );
    let shift_counts = _mm256_setr_epi64x(shifts[0], shifts[1], shifts[2], shifts[3]);
    let inverse_shift_counts = _mm256_setr_epi64x(
        inverse_shifts[0],
        inverse_shifts[1],
        inverse_shifts[2],
        inverse_shifts[3],
    );
    let lower_words = unsafe { _mm256_i64gather_epi64::<8>(base, low_indices) };
    let upper_words = unsafe { _mm256_i64gather_epi64::<8>(base, high_indices) };
    let lower_bits = _mm256_srlv_epi64(lower_words, shift_counts);
    let upper_bits = _mm256_sllv_epi64(upper_words, inverse_shift_counts);
    let candidate_windows = _mm256_or_si256(lower_bits, upper_bits);
    let mask = _mm256_set1_epi64x(needle_first_word_mask as i64);
    let needle = _mm256_set1_epi64x(needle_first_word as i64);
    let candidates = _mm256_and_si256(candidate_windows, mask);
    let equal = _mm256_cmpeq_epi64(candidates, needle);
    let lanes = unsafe { _mm256_movemask_pd(core::mem::transmute(equal)) } as u8;
    Some(lanes)
}
