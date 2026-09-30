use crate::traits::{WordsUnpack, layout_block_len};

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Collects the decoded characters into a vector.
    pub fn to_vec(&self) -> alloc::vec::Vec<C> {
        // 64 codes is a multiple of every layout block for BITS=1..=8.
        const CODE_BATCH_LEN: usize = 64;

        debug_assert!(CODE_BATCH_LEN.is_multiple_of(layout_block_len::<BITS>()));

        let char_len = self.char_len();
        if char_len < CODE_BATCH_LEN {
            return self.iter().collect();
        }

        let bulk_code_len = char_len / CODE_BATCH_LEN * CODE_BATCH_LEN;
        let words = self.bits.words();
        let words_per_batch = CODE_BATCH_LEN * usize::from(BITS) / crate::WORD_BITS;
        let mut decoded = [0u8; CODE_BATCH_LEN];
        let mut result = alloc::vec::Vec::with_capacity(char_len);

        for batch_index in 0..bulk_code_len / CODE_BATCH_LEN {
            let word_start = batch_index * words_per_batch;
            let word_end = word_start + words_per_batch;
            words[word_start..word_end].unpack_codes::<BITS>(&mut decoded);

            result.extend(decoded.iter().map(|&code| {
                C::from_code(code).expect("PackedChar rejected a code it previously produced")
            }));
        }

        result.extend((bulk_code_len..char_len).map(|index| {
            self.get(index)
                .expect("packed character index is within the string")
        }));

        result
    }
}

#[cfg(test)]
mod tests_for_conversion;
