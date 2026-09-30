use crate::WORD_BITS;

pub(super) fn unpack_codes<const BITS: u8>(words: &[u64], codes: &mut [u8]) {
    let width = usize::from(BITS);
    let mask = (1u64 << BITS) - 1;
    let mut next_word_index = 0;
    let mut pending_bits = 0u64;
    let mut pending_bit_count = 0;

    for code in codes {
        let value = if pending_bit_count >= width {
            let value = pending_bits;
            pending_bits >>= width;
            pending_bit_count -= width;
            value
        } else {
            let old_fill = pending_bit_count;
            let next_word = words[next_word_index];
            next_word_index += 1;

            let value = pending_bits | (next_word << old_fill);
            let used_from_next = width - old_fill;
            pending_bits = next_word >> used_from_next;
            pending_bit_count = WORD_BITS - used_from_next;
            value
        };

        *code = (value & mask) as u8;
    }
}
