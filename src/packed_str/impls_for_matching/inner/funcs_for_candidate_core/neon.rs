use core::arch::aarch64::{
    uint8x16x2_t, uint64x2_t, vandq_u64, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_s64,
    vld1q_u8, vld1q_u64, vorrq_u64, vqtbl2q_u8, vreinterpretq_u64_u8, vshlq_u64,
};

use crate::{BitStr, WORD_BITS};

#[target_feature(enable = "neon")]
pub(super) unsafe fn matching_lanes(
    windows: &[u64; 4],
    needle_first_word: u64,
    needle_first_word_mask: u64,
) -> u8 {
    let needle = unsafe { vdupq_n_u64(needle_first_word) };
    let mask = unsafe { vdupq_n_u64(needle_first_word_mask) };
    let lower = unsafe { vld1q_u64(windows.as_ptr()) };
    let upper = unsafe { vld1q_u64(windows.as_ptr().add(2)) };
    let lower = unsafe { vceqq_u64(vandq_u64(lower, mask), needle) };
    let upper = unsafe { vceqq_u64(vandq_u64(upper, mask), needle) };
    let mut matching_lanes = 0;
    if unsafe { vgetq_lane_u64(lower, 0) } != 0 {
        matching_lanes |= 1;
    }
    if unsafe { vgetq_lane_u64(lower, 1) } != 0 {
        matching_lanes |= 1 << 1;
    }
    if unsafe { vgetq_lane_u64(upper, 0) } != 0 {
        matching_lanes |= 1 << 2;
    }
    if unsafe { vgetq_lane_u64(upper, 1) } != 0 {
        matching_lanes |= 1 << 3;
    }
    matching_lanes
}

#[target_feature(enable = "neon")]
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
        .checked_add(3)
        .is_none_or(|last_word| last_word >= source_words.len())
    {
        return None;
    }

    let source_bytes = unsafe { source_words.as_ptr().add(base_word).cast::<u8>() };
    let table = uint8x16x2_t(unsafe { vld1q_u8(source_bytes) }, unsafe {
        vld1q_u8(source_bytes.add(16))
    });
    let mut word_offsets = [0usize; 4];
    let mut shifts = [0i64; 4];
    for lane in 0..4 {
        let lane_bit = first_shift + lane * width;
        word_offsets[lane] = lane_bit / WORD_BITS;
        shifts[lane] = (lane_bit % WORD_BITS) as i64;
    }

    let lower_words = unsafe { gather_words(table, [word_offsets[0], word_offsets[1]]) };
    let lower_upper_words =
        unsafe { gather_words(table, [word_offsets[0] + 1, word_offsets[1] + 1]) };
    let upper_words = unsafe { gather_words(table, [word_offsets[2], word_offsets[3]]) };
    let upper_upper_words =
        unsafe { gather_words(table, [word_offsets[2] + 1, word_offsets[3] + 1]) };

    let lower_window = unsafe {
        make_windows(
            lower_words,
            lower_upper_words,
            [-shifts[0], -shifts[1]],
            [64 - shifts[0], 64 - shifts[1]],
        )
    };
    let upper_window = unsafe {
        make_windows(
            upper_words,
            upper_upper_words,
            [-shifts[2], -shifts[3]],
            [64 - shifts[2], 64 - shifts[3]],
        )
    };

    let needle = vdupq_n_u64(needle_first_word);
    let mask = vdupq_n_u64(needle_first_word_mask);
    let lower_matches = vceqq_u64(vandq_u64(lower_window, mask), needle);
    let upper_matches = vceqq_u64(vandq_u64(upper_window, mask), needle);
    let mut matching_lanes = 0;
    if unsafe { vgetq_lane_u64(lower_matches, 0) } != 0 {
        matching_lanes |= 1;
    }
    if unsafe { vgetq_lane_u64(lower_matches, 1) } != 0 {
        matching_lanes |= 1 << 1;
    }
    if unsafe { vgetq_lane_u64(upper_matches, 0) } != 0 {
        matching_lanes |= 1 << 2;
    }
    if unsafe { vgetq_lane_u64(upper_matches, 1) } != 0 {
        matching_lanes |= 1 << 3;
    }
    Some(matching_lanes)
}

#[target_feature(enable = "neon")]
unsafe fn gather_words(table: uint8x16x2_t, word_offsets: [usize; 2]) -> uint64x2_t {
    let mut byte_indices = [0u8; 16];
    for (lane, word_offset) in word_offsets.into_iter().enumerate() {
        for byte in 0..8 {
            byte_indices[lane * 8 + byte] = (word_offset * 8 + byte) as u8;
        }
    }
    let indices = unsafe { vld1q_u8(byte_indices.as_ptr()) };
    vreinterpretq_u64_u8(vqtbl2q_u8(table, indices))
}

#[target_feature(enable = "neon")]
unsafe fn make_windows(
    lower_words: uint64x2_t,
    upper_words: uint64x2_t,
    right_shift: [i64; 2],
    left_shift: [i64; 2],
) -> uint64x2_t {
    let right_shift = unsafe { vld1q_s64(right_shift.as_ptr()) };
    let left_shift = unsafe { vld1q_s64(left_shift.as_ptr()) };
    vorrq_u64(
        vshlq_u64(lower_words, right_shift),
        vshlq_u64(upper_words, left_shift),
    )
}
