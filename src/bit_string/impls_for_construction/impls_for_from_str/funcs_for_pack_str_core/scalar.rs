/// Validate-and-pack: process 64 bytes at a time.
///
/// # Safety
///
/// - `src` must be valid for reads of `bit_len` u8 values.
/// - `dst` must be valid for writes of `ceil(bit_len / 64)` u64 values.
#[inline]
pub(super) unsafe fn words(
    mut dst: *mut u64,
    mut src: *const u8,
    mut bit_len: usize,
) -> Option<(usize, u8)> {
    let mut global_offset = 0usize;

    while bit_len >= 64 {
        let mut word = 0u64;
        for i in 0..64 {
            // SAFETY: `i < 64` and `bit_len >= 64`.
            let byte = unsafe { src.add(i).read() };
            let bit = match byte {
                b'0' => 0u64,
                b'1' => 1u64,
                _ => return Some((global_offset + i, byte)),
            };
            word |= bit << i;
        }

        // SAFETY: `dst` points to the next output slot.
        unsafe {
            *dst = word;
            dst = dst.add(1);
            src = src.add(64);
        }
        global_offset += 64;
        bit_len -= 64;
    }

    if bit_len > 0 {
        let mut word = 0u64;
        for i in 0..bit_len {
            // SAFETY: `i < bit_len`.
            let byte = unsafe { src.add(i).read() };
            let bit = match byte {
                b'0' => 0u64,
                b'1' => 1u64,
                _ => return Some((global_offset + i, byte)),
            };
            word |= bit << i;
        }

        // SAFETY: `dst` points to the last output slot.
        unsafe {
            *dst = word;
        }
    }

    None
}
