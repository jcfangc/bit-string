use crate::WORD_BITS;

pub(super) fn unpack_codes<const BITS: u8>(words: &[u64], codes: &mut [u8]) {
    let width = usize::from(BITS);
    let mask = (1u64 << BITS) - 1;

    for (index, code) in codes.iter_mut().enumerate() {
        let bit_position = index * width;
        let word_index = bit_position / WORD_BITS;
        let bit_offset = bit_position % WORD_BITS;
        let mut value = words[word_index] >> bit_offset;

        if bit_offset + width > WORD_BITS {
            value |= words[word_index + 1] << (WORD_BITS - bit_offset);
        }

        *code = (value & mask) as u8;
    }
}
