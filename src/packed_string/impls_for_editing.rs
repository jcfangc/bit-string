use super::funcs_for_code::{PACK_CODE_BATCH_LEN, for_each_aligned_code_batch};
use super::*;
use crate::traits::WordsPack;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Appends the character's code directly to the payload.
    pub fn push(&mut self, character: C) {
        let code = character.code();
        if BITS == 8 {
            self.bits.push_byte(code);
        } else {
            self.bits.push_bits(u64::from(code), usize::from(BITS));
        }
    }

    pub fn pop(&mut self) -> Option<C> {
        let character = self.last()?;
        self.bits
            .truncate((self.char_len() - 1) * usize::from(BITS));
        Some(character)
    }

    pub fn set(&mut self, index: usize, character: C) -> Option<C> {
        let previous = self.get(index)?;
        write_code::<BITS>(&mut self.bits, index, character.code());
        Some(previous)
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.char_len() {
            return;
        }
        self.bits.truncate(new_len * usize::from(BITS));
    }

    pub fn clear(&mut self) {
        self.bits.clear();
    }
}

impl<C, const BITS: u8> Extend<C> for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn extend<I: IntoIterator<Item = C>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let width = usize::from(BITS);
        let (lower_bound, upper_bound) = iter.size_hint();

        if let Some(additional_bits) = lower_bound.checked_mul(width) {
            self.bits.reserve_bits(additional_bits);
        }

        // Avoid batch setup when the iterator can yield fewer than one batch.
        if matches!(upper_bound, Some(upper_bound) if upper_bound < PACK_CODE_BATCH_LEN) {
            for character in iter {
                self.push(character);
            }
            return;
        }

        let mut packed_words = [0u64; PACK_CODE_BATCH_LEN / 8];
        let bits = &mut self.bits;
        // Keep scratch space bounded; collecting the whole iterator first would
        // require a temporary PackedString for the complete suffix.
        // If iteration panics, completed batches remain appended and the partial
        // batch is discarded during unwinding.
        let (tail_codes, tail_len) = for_each_aligned_code_batch::<C, BITS, _, _>(iter, |codes| {
            let packed_bit_len = codes.len() * width;
            let packed_word_len = packed_bit_len / crate::WORD_BITS;
            packed_words[..packed_word_len].pack_codes::<BITS>(codes);
            bits.push_packed_words(&packed_words[..packed_word_len], packed_bit_len);
        });

        for &code in &tail_codes[..tail_len] {
            if BITS == 8 {
                self.bits.push_byte(code);
            } else {
                self.bits.push_bits(u64::from(code), width);
            }
        }
    }
}

impl<'a, C, const BITS: u8> Extend<&'a C> for PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    fn extend<I: IntoIterator<Item = &'a C>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied());
    }
}

#[cfg(test)]
mod tests_for_editing;

mod impls_for_concat;
mod impls_for_drain;
mod impls_for_insert_remove;
mod impls_for_replace;
mod impls_for_retain;
mod impls_for_reverse;
mod impls_for_slice;
