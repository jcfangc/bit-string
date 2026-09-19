use crate::code_mask;

/// Packs complete layout blocks into complete little-endian words.
pub(super) fn pack_codes<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    if BITS == 8 {
        pack_bytes(dst, codes);
        return;
    }

    let width = usize::from(BITS);
    let mask = u64::from(code_mask::<BITS>());

    dst.fill(0);

    let mut word_index = 0;
    let mut bit_offset = 0;
    for &code in codes {
        let code = u64::from(code) & mask;
        dst[word_index] |= code << bit_offset;

        let next_offset = bit_offset + width;
        if next_offset > 64 {
            dst[word_index + 1] |= code >> (64 - bit_offset);
            word_index += 1;
            bit_offset = next_offset - 64;
        } else if next_offset == 64 {
            word_index += 1;
            bit_offset = 0;
        } else {
            bit_offset = next_offset;
        }
    }
}

#[inline]
fn pack_bytes(dst: &mut [u64], codes: &[u8]) {
    debug_assert_eq!(codes.len(), dst.len() * 8);

    for (word, chunk) in dst.iter_mut().zip(codes.chunks_exact(8)) {
        *word = u64::from_le_bytes(chunk.try_into().expect("chunk has eight bytes"));
    }
}
