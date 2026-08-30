use super::scalar;

use core::arch::aarch64::{
    vand_u8, vceq_u8, vdup_n_u8, vget_lane_u64, vld1_u8, vpaddl_u8, vpaddl_u16, vpaddl_u32,
};

/// Bit-position masks: [1, 2, 4, 8, 16, 32, 64, 128].
const BIT_MASKS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

/// NEON backend: packs 64 bytes at a time into one u64.
///
/// Each 8-byte group contributes bits at `group * 8` offset within the
/// current word, then 8 groups are OR'd together.
///
/// # Safety
///
/// - Caller must only call this when NEON is available.
/// - `src` must be valid for reads of `bit_len` u8 values.
/// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
#[target_feature(enable = "neon")]
pub(super) unsafe fn words(mut dst: *mut u64, mut src: *const u8, mut bit_len: usize) {
    // SAFETY: constant pointer to static mask array.
    let bit_masks = unsafe { vld1_u8(BIT_MASKS.as_ptr()) };
    let ones = vdup_n_u8(1);

    while bit_len >= 64 {
        // Accumulate 8 groups × 8 bytes → one u64.
        let mut word = 0u64;
        for group in 0..8 {
            // SAFETY: `bit_len >= 64` and `group < 8`, so the load
            // is within bounds. `vld1_u8` permits unaligned reads.
            let bytes = unsafe { vld1_u8(src.add(group * 8)) };

            // Expand 1 → 0xFF (0 stays 0x00), then mask to position
            // each bit before reducing to a single u64 via pairwise adds.
            let is_one = vceq_u8(bytes, ones);
            let masked = vand_u8(is_one, bit_masks);
            let sum16 = vpaddl_u8(masked);
            let sum32 = vpaddl_u16(sum16);
            let sum64 = vpaddl_u32(sum32);

            let group_bits = vget_lane_u64::<0>(sum64);
            word |= group_bits << (group * 8);
        }

        // SAFETY: `dst` points to the current output slot; destination
        // has capacity for this write per the caller's contract.
        unsafe {
            *dst = word;
            dst = dst.add(1);
            src = src.add(64);
        }
        bit_len -= 64;
    }

    // SAFETY: `bit_len < 64`, delegate tail to scalar.
    unsafe {
        scalar::words(dst, src, bit_len);
    }
}
