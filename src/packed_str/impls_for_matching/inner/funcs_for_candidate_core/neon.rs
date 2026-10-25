use core::arch::aarch64::{vandq_u64, vceqq_u64, vdupq_n_u64, vgetq_lane_u64, vld1q_u64};

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
