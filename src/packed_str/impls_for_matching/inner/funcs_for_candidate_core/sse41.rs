#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_extract_epi64, _mm_loadu_si128, _mm_movemask_epi8,
    _mm_set1_epi64x,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_extract_epi64, _mm_loadu_si128, _mm_movemask_epi8,
    _mm_set1_epi64x,
};

use crate::{BitStr, WORD_BITS};

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

#[target_feature(enable = "sse4.1")]
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
    if base_word
        .checked_add(2)
        .is_none_or(|last_word| last_word >= source_words.len())
    {
        return None;
    }

    let source_pair = unsafe { _mm_loadu_si128(source_words.as_ptr().add(base_word).cast()) };
    let source_words = [
        unsafe { _mm_extract_epi64::<0>(source_pair) as u64 },
        unsafe { _mm_extract_epi64::<1>(source_pair) as u64 },
        source_words[base_word + 2],
    ];
    let mut windows = [0u64; 4];
    for (lane, window) in windows.iter_mut().enumerate() {
        let lane_start = first_shift + lane * width;
        let word_offset = lane_start / WORD_BITS;
        let shift = lane_start % WORD_BITS;
        let lower = source_words[word_offset];
        *window = if shift == 0 {
            lower
        } else {
            (lower >> shift) | (source_words[word_offset + 1] << (WORD_BITS - shift))
        };
    }

    Some(unsafe { matching_lanes(&windows, needle_first_word, needle_first_word_mask) })
}
