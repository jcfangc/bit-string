use super::*;
use crate::traits::{WordsPack, layout_block_len};

const RETAIN_BATCH_LEN: usize = 64;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Retains the characters for which `predicate` returns `true`.
    ///
    /// The predicate is called once per character, in order. Retained
    /// characters keep their relative order.
    ///
    /// If `predicate` panics, `self` remains a valid packed string, but its
    /// contents may have been partially compacted. The original contents are
    /// not restored.
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(C) -> bool,
    {
        let original_len = self.char_len();
        let mut write = 0usize;
        let mut retained_codes = [0u8; RETAIN_BATCH_LEN];
        let mut retained_len = 0usize;

        for read in 0..original_len {
            let character = self.get(read).unwrap();
            if predicate(character) {
                retained_codes[retained_len] = character.code();
                retained_len += 1;

                if retained_len == RETAIN_BATCH_LEN {
                    if write + retained_len != read + 1 {
                        write_packed_codes::<BITS>(&mut self.bits, write, &retained_codes);
                    }
                    write += retained_len;
                    retained_len = 0;
                }
            }
        }

        let mut final_len = write + retained_len;
        if final_len != original_len {
            let layout_len = layout_block_len::<BITS>();
            let aligned_len = retained_len / layout_len * layout_len;
            if aligned_len != 0 {
                write_packed_codes::<BITS>(&mut self.bits, write, &retained_codes[..aligned_len]);
                write += aligned_len;
            }

            let width = usize::from(BITS);
            for &code in &retained_codes[aligned_len..retained_len] {
                self.bits.set_chunk(write * width, u64::from(code), width);
                write += 1;
            }

            final_len = write;
        }

        self.truncate(final_len);
    }
}

fn write_packed_codes<const BITS: u8>(bits: &mut crate::BitString, start: usize, codes: &[u8]) {
    let width = usize::from(BITS);
    let bit_len = codes.len() * width;
    debug_assert!(bit_len.is_multiple_of(crate::WORD_BITS));

    let mut words = [0u64; RETAIN_BATCH_LEN * 8 / crate::WORD_BITS];
    let word_len = bit_len / crate::WORD_BITS;
    words[..word_len].pack_codes::<BITS>(codes);

    let bit_start = start * width;
    for (word_index, &word) in words[..word_len].iter().enumerate() {
        bits.set_chunk(
            bit_start + word_index * crate::WORD_BITS,
            word,
            crate::WORD_BITS,
        );
    }
}

#[cfg(test)]
mod tests_for_retain;
