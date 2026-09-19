use crate::code_mask;

/// Packs complete layout blocks into complete little-endian words.
pub(super) fn pack_codes<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    if BITS == 8 {
        pack_bytes(dst, codes);
        return;
    }
    if BITS == 4 {
        pack_nibbles(dst, codes);
        return;
    }
    if BITS == 2 {
        pack_pairs(dst, codes);
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

#[inline]
fn pack_nibbles(dst: &mut [u64], codes: &[u8]) {
    debug_assert_eq!(codes.len(), dst.len() * 16);

    for (word, chunk) in dst.iter_mut().zip(codes.chunks_exact(16)) {
        let mut packed = 0u64;
        for (index, &code) in chunk.iter().enumerate() {
            packed |= u64::from(code & 0x0f) << (index * 4);
        }
        *word = packed;
    }
}

#[inline]
fn pack_pairs(dst: &mut [u64], codes: &[u8]) {
    debug_assert_eq!(codes.len(), dst.len() * 32);

    for (word, chunk) in dst.iter_mut().zip(codes.chunks_exact(32)) {
        let mut packed = 0u64;
        for (index, &code) in chunk.iter().enumerate() {
            packed |= u64::from(code & 0x03) << (index * 2);
        }
        *word = packed;
    }
}
