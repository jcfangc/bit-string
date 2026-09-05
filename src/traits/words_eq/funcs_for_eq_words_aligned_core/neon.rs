use core::arch::aarch64::{uint64x2_t, vceqq_u64, vgetq_lane_u64, vld1q_u64};

#[target_feature(enable = "neon")]
pub(super) unsafe fn eq_words(src: &[u64], other: &[u64], len: usize) -> bool {
    let mut i = 0;
    while i + 2 <= len {
        // SAFETY: `#[target_feature(enable = "neon")]` ensures NEON is enabled.
        // Pointers `src` and `other` are valid for `len` elements (guaranteed by caller).
        let a = unsafe { vld1q_u64(src.as_ptr().add(i)) };
        // SAFETY: same as above; load from `other`.
        let b = unsafe { vld1q_u64(other.as_ptr().add(i)) };
        // SAFETY: `vceqq_u64` and `vgetq_lane_u64` are pure register operations; NEON is enabled by `#[target_feature]`.
        let cmp = unsafe { vceqq_u64(a, b) };
        // SAFETY: same as above
        if unsafe { vgetq_lane_u64(cmp, 0) } == 0 || unsafe { vgetq_lane_u64(cmp, 1) } == 0 {
            return false;
        }
        i += 2;
    }
    while i < len {
        if src[i] != other[i] {
            return false;
        }
        i += 1;
    }
    true
}
