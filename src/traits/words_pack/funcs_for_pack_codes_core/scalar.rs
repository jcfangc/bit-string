/// Packs complete layout blocks into complete little-endian words.
pub(super) fn pack_codes<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    match BITS {
        8 => pack_bytes(dst, codes),
        4 => pack_nibbles(dst, codes),
        2 => pack_pairs(dst, codes),
        1 | 3 | 5 | 6 | 7 => pack_groups::<BITS>(dst, codes),
        _ => unreachable!("BITS validated by WordsPack core"),
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
            packed |= u64::from(code) << (index * 4);
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
            packed |= u64::from(code) << (index * 2);
        }
        *word = packed;
    }
}

#[inline]
fn pack_groups<const BITS: u8>(dst: &mut [u64], codes: &[u8]) {
    let width = usize::from(BITS);
    let group_bits = width * 8;
    let mut accumulator = 0u64;
    let mut accumulated_bits = 0;
    let mut word_index = 0;

    for group in codes.chunks_exact(8) {
        let packed_group = pack_group::<BITS>(group);

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

#[inline]
fn pack_group<const BITS: u8>(group: &[u8]) -> u64 {
    if BITS == 6 {
        return u64::from(group[0])
            | (u64::from(group[1]) << 6)
            | (u64::from(group[2]) << 12)
            | (u64::from(group[3]) << 18)
            | (u64::from(group[4]) << 24)
            | (u64::from(group[5]) << 30)
            | (u64::from(group[6]) << 36)
            | (u64::from(group[7]) << 42);
    }

    let width = usize::from(BITS);
    let mut packed = 0u64;
    for (index, &code) in group.iter().enumerate() {
        packed |= u64::from(code) << (index * width);
    }
    packed
}
