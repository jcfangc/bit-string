use super::scalar;

use core::arch::aarch64::{
    vand_u8, vceq_u8, vdup_n_u8, veor_u8, vget_lane_u64, vld1_u8, vpaddl_u8, vpaddl_u16,
    vpaddl_u32, vreinterpret_u64_u8,
};

/// Bit-position masks for vpaddl reduction.
const BIT_MASKS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

/// NEON backend: validate + pack, 64 bytes at a time.
///
/// Uses `(v ^ 0x30) & (v ^ 0x31)` for validation, reusing `v ^ 0x30`
/// (which holds the bit value in LSB) for extraction.
#[target_feature(enable = "neon")]
pub(super) unsafe fn words(
    mut dst: *mut u64,
    mut src: *const u8,
    mut bit_len: usize,
) -> Option<(usize, u8)> {
    let zero_byte = vdup_n_u8(b'0');
    let one_byte = vdup_n_u8(b'1');
    // SAFETY: `BIT_MASKS` is a static array of 8 u8 values, so its pointer is
    // valid for reads of 8 bytes.
    let bit_masks = unsafe { vld1_u8(BIT_MASKS.as_ptr()) };
    let mut global_offset = 0usize;

    while bit_len >= 64 {
        // Validate all 8 groups, then pack them into one u64.
        let mut word = 0u64;
        for group in 0..8 {
            // SAFETY: `bit_len >= 64` and `group < 8`, so the
            // 8-byte read from `src + group*8` is in bounds.
            let v = unsafe { vld1_u8(src.add(group * 8)) };

            // `xor0 = v ^ 0x30`:
            //   b'0' → 0x00, b'1' → 0x01  (bit value in LSB)
            // `xor1 = v ^ 0x31`:
            //   b'0' → 0x01, b'1' → 0x00
            // `invalid = xor0 & xor1`:
            //   valid → 0x00, invalid → non-zero
            let xor0 = veor_u8(v, zero_byte);
            let xor1 = veor_u8(v, one_byte);
            let invalid = vand_u8(xor0, xor1);

            // Cheap all-zero test via reinterpret as u64.
            let invalid_u64 = vget_lane_u64::<0>(vreinterpret_u64_u8(invalid));
            if invalid_u64 != 0 {
                // Fall back to scalar for exact error position within
                // this 64-byte chunk.
                // SAFETY: caller guarantees pointer validity. Scalar backend is always safe.
                let (i, b) =
                    unsafe { scalar::words(dst, src, 64) }.expect("chunk has invalid byte");
                return Some((global_offset + i, b));
            }

            // Pack: xor0 holds the bit value (0x00 or 0x01).
            // Expand 0x01 → 0xFF via vceq_u8 so that vand with
            // bit-position masks correctly captures every lane,
            // then horizontal pairwise add collapses to one u64.
            let is_one = vceq_u8(xor0, vdup_n_u8(1));
            let masked = vand_u8(is_one, bit_masks);
            let sum16 = vpaddl_u8(masked);
            let sum32 = vpaddl_u16(sum16);
            let sum64 = vpaddl_u32(sum32);

            let group_bits = vget_lane_u64::<0>(sum64);
            word |= group_bits << (group * 8);
        }

        // SAFETY: `dst` points to the current output slot.
        unsafe {
            *dst = word;
            dst = dst.add(1);
            src = src.add(64);
        }
        global_offset += 64;
        bit_len -= 64;
    }

    // SAFETY: `bit_len < 64`.
    unsafe {
        if let Some((i, b)) = scalar::words(dst, src, bit_len) {
            return Some((global_offset + i, b));
        }
    }
    None
}
