use alloc::vec::Vec;

use crate::{BitString, WORD_BITS, code_mask, word_len};

use super::*;

pub(super) fn pack_codes<C, const BITS: u8, I>(chars: I) -> BitString
where
    C: PackedChar<BITS>,
    I: IntoIterator<Item = C>,
{
    let width = usize::from(BITS);
    let value_mask = u64::from(code_mask::<BITS>());
    let chars = chars.into_iter();
    let (lower_bound, _) = chars.size_hint();
    let mut words = Vec::new();
    if let Some(bit_len) = lower_bound.checked_mul(width) {
        words.reserve(word_len(bit_len));
    }

    let mut bit_len = 0_usize;
    for character in chars {
        let new_bit_len = bit_len
            .checked_add(width)
            .expect("packed string length overflow");
        let word_index = bit_len / WORD_BITS;
        let bit_offset = bit_len % WORD_BITS;

        if word_index == words.len() {
            words.push(0);
        }

        let code = u64::from(character.code()) & value_mask;
        words[word_index] |= code << bit_offset;

        let next_offset = bit_offset + width;
        if next_offset > WORD_BITS {
            if word_index + 1 == words.len() {
                words.push(0);
            }
            words[word_index + 1] |= code >> (WORD_BITS - bit_offset);
        }

        bit_len = new_bit_len;
    }

    BitString::from_words(&words, bit_len).expect("packed words have an invalid length")
}

#[inline]
pub(super) fn write_code<const BITS: u8>(bits: &mut BitString, position: usize, code: u8) {
    let start = position * usize::from(BITS);
    bits.set_chunk(start, u64::from(code), usize::from(BITS));
}
