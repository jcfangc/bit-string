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
    if BITS == 6 {
        pack_six_bits(dst, codes);
        return;
    }
    if matches!(BITS, 1 | 3 | 5 | 7) {
        pack_groups::<BITS>(dst, codes);
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

#[inline]
fn pack_six_bits(dst: &mut [u64], codes: &[u8]) {
    debug_assert_eq!(codes.len() % 8, 0);

    let mut accumulator = 0u64;
    let mut accumulated_bits = 0;
    let mut word_index = 0;

    for group in codes.chunks_exact(8) {
        let packed_group = u64::from(group[0] & 0x3f)
            | (u64::from(group[1] & 0x3f) << 6)
            | (u64::from(group[2] & 0x3f) << 12)
            | (u64::from(group[3] & 0x3f) << 18)
            | (u64::from(group[4] & 0x3f) << 24)
            | (u64::from(group[5] & 0x3f) << 30)
            | (u64::from(group[6] & 0x3f) << 36)
            | (u64::from(group[7] & 0x3f) << 42);
        let next_bits = accumulated_bits + 48;

        if next_bits < 64 {
            accumulator |= packed_group << accumulated_bits;
            accumulated_bits = next_bits;
        } else if next_bits == 64 {
            dst[word_index] = accumulator | (packed_group << accumulated_bits);
            word_index += 1;
            accumulator = 0;
            accumulated_bits = 0;
        } else {
            dst[word_index] = accumulator | (packed_group << accumulated_bits);
            word_index += 1;
            accumulator = packed_group >> (64 - accumulated_bits);
            accumulated_bits = next_bits - 64;
        }
    }

    debug_assert_eq!(accumulated_bits, 0);
    debug_assert_eq!(word_index, dst.len());
}

#[inline]
fn pack_groups<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    let width = usize::from(BITS);
    let mask = u64::from(code_mask::<BITS>());
    let group_bits = width * 8;
    let mut accumulator = 0u64;
    let mut accumulated_bits = 0;
    let mut word_index = 0;

    for group in codes.chunks_exact(8) {
        let mut packed_group = 0u64;
        for (index, &code) in group.iter().enumerate() {
            packed_group |= (u64::from(code) & mask) << (index * width);
        }

        let next_bits = accumulated_bits + group_bits;
        if next_bits < 64 {
            accumulator |= packed_group << accumulated_bits;
            accumulated_bits = next_bits;
        } else if next_bits == 64 {
            dst[word_index] = accumulator | (packed_group << accumulated_bits);
            word_index += 1;
            accumulator = 0;
            accumulated_bits = 0;
        } else {
            dst[word_index] = accumulator | (packed_group << accumulated_bits);
            word_index += 1;
            accumulator = packed_group >> (64 - accumulated_bits);
            accumulated_bits = next_bits - 64;
        }
    }

    debug_assert_eq!(accumulated_bits, 0);
    debug_assert_eq!(word_index, dst.len());
}
