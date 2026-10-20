use alloc::vec::Vec;

use crate::traits::{WordsPack, layout_block_len};
use crate::{BitString, WORD_BITS, word_len};

use super::*;

// Encoding batch size shared by construction and extension. A batch contains
// complete layout blocks for BITS=1..=8; tune independently of decoding batches.
pub(super) const PACK_CODE_BATCH_LEN: usize = 64;

pub(super) fn pack_codes<C, const BITS: u8, I>(chars: I) -> BitString
where
    C: PackedChar<BITS>,
    I: IntoIterator<Item = C>,
{
    let width = usize::from(BITS);
    let chars = chars.into_iter();
    let (lower_bound, _) = chars.size_hint();
    let mut words = Vec::new();
    if let Some(bit_len) = lower_bound.checked_mul(width) {
        words.reserve(word_len(bit_len));
    }

    let (tail_codes, tail_len) = for_each_aligned_code_batch::<C, BITS, _, _>(chars, |codes| {
        let start = words.len();
        words.resize(start + codes.len() * width / WORD_BITS, 0);
        words[start..].pack_codes::<BITS>(codes);
    });
    let mut bit_len = words.len() * WORD_BITS;
    for &code in &tail_codes[..tail_len] {
        append_code::<BITS>(&mut words, &mut bit_len, u64::from(code));
    }

    BitString::from_owned_words(words, bit_len).expect("packed words have an invalid length")
}

/// Passes full code batches and final complete layout blocks to `consume`.
/// Returns any remaining codes that cannot form a word-aligned layout block.
///
/// Construction and extension share this batching so extension can pack a
/// bounded batch at a time instead of materializing the entire input first.
pub(super) fn for_each_aligned_code_batch<C, const BITS: u8, I, F>(
    chars: I,
    mut consume: F,
) -> ([u8; PACK_CODE_BATCH_LEN], usize)
where
    C: PackedChar<BITS>,
    I: IntoIterator<Item = C>,
    F: FnMut(&[u8]),
{
    let layout_len = layout_block_len::<BITS>();
    let mut batch = [0u8; PACK_CODE_BATCH_LEN];
    let mut batch_len = 0;

    for character in chars {
        batch[batch_len] = character.code();
        batch_len += 1;

        if batch_len == PACK_CODE_BATCH_LEN {
            consume(&batch);
            batch_len = 0;
        }
    }

    let aligned_count = batch_len / layout_len * layout_len;
    if aligned_count != 0 {
        consume(&batch[..aligned_count]);
    }

    let tail_len = batch_len - aligned_count;
    batch.copy_within(aligned_count..batch_len, 0);
    (batch, tail_len)
}

#[inline]
fn append_code<const BITS: u8>(words: &mut Vec<u64>, bit_len: &mut usize, code: u64) {
    let width = usize::from(BITS);
    let new_bit_len = bit_len
        .checked_add(width)
        .expect("packed string length overflow");
    let word_index = *bit_len / WORD_BITS;
    let bit_offset = *bit_len % WORD_BITS;

    if word_index == words.len() {
        words.push(0);
    }

    words[word_index] |= code << bit_offset;

    let next_offset = bit_offset + width;
    if next_offset > WORD_BITS {
        if word_index + 1 == words.len() {
            words.push(0);
        }
        words[word_index + 1] |= code >> (WORD_BITS - bit_offset);
    }

    *bit_len = new_bit_len;
}

#[inline]
pub(super) fn write_code<const BITS: u8>(bits: &mut BitString, position: usize, code: u8) {
    let start = position * usize::from(BITS);
    bits.set_chunk(start, u64::from(code), usize::from(BITS));
}
