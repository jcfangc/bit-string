use core::cmp::Ordering;

use crate::traits::WordOrd;
use core::arch::aarch64::{uint64x2_t, vceqq_u64, vgetq_lane_u64, vld1q_u64};

#[target_feature(enable = "neon")]
pub(super) unsafe fn cmp_aligned(src: &[u64], other: &[u64], len: usize) -> Option<Ordering> {
    let mut i = 0;
    while i + 2 <= len {
        let a = unsafe { vld1q_u64(src.as_ptr().add(i)) };
        let b = unsafe { vld1q_u64(other.as_ptr().add(i)) };
        let cmp = unsafe { vceqq_u64(a, b) };
        if unsafe { vgetq_lane_u64(cmp, 0) } == 0 {
            return Some(WordOrd::bitwise_cmp(src[i], other[i]));
        }
        if unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
            return Some(WordOrd::bitwise_cmp(src[i + 1], other[i + 1]));
        }
        i += 2;
    }
    while i < len {
        if src[i] != other[i] {
            return Some(WordOrd::bitwise_cmp(src[i], other[i]));
        }
        i += 1;
    }
    None
}
