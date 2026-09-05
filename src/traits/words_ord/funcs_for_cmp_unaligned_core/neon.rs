use core::cmp::Ordering;

use crate::WORD_BITS;
use crate::traits::WordOrd;

use core::arch::aarch64::{
    vceqq_u64, vdupq_n_s64, vgetq_lane_u64, vld1q_u64, vorrq_u64, vshlq_u64,
};

#[target_feature(enable = "neon")]
pub(super) unsafe fn cmp_unaligned(
    src: &[u64],
    other: &[u64],
    len: usize,
    shift: usize,
) -> Option<Ordering> {
    let neg_shift = unsafe { vdupq_n_s64(-(shift as i64)) };
    let pos_shift = unsafe { vdupq_n_s64((WORD_BITS - shift) as i64) };
    let mut i = 0;
    while i + 2 <= len {
        let w0 = unsafe { vld1q_u64(src.as_ptr().add(i)) };
        let w1 = unsafe { vld1q_u64(src.as_ptr().add(i + 1)) };
        let lo = unsafe { vshlq_u64(w0, neg_shift) };
        let hi = unsafe { vshlq_u64(w1, pos_shift) };
        let window = unsafe { vorrq_u64(lo, hi) };
        let expected = unsafe { vld1q_u64(other.as_ptr().add(i)) };
        let cmp = unsafe { vceqq_u64(window, expected) };
        if unsafe { vgetq_lane_u64(cmp, 0) } == 0 {
            let sw = (src[i] >> shift) | (src[i + 1] << (WORD_BITS - shift));
            return Some(WordOrd::bitwise_cmp(sw, other[i]));
        }
        if unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
            let sw = (src[i + 1] >> shift) | (src[i + 2] << (WORD_BITS - shift));
            return Some(WordOrd::bitwise_cmp(sw, other[i + 1]));
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
