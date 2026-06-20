use crate::bit_string::traits::*;
use crate::funcs_for_bits::*;

use super::*;

/// Compare `needle` bits against `haystack` starting at `offset`, using
/// word-level reads so that each iteration compares up to 64 bits.
#[inline]
fn bits_equal_at(haystack: &BitString, offset: usize, needle: &BitString) -> bool {
    let needle_bits = needle.bit_len;
    let needle_words = needle.as_words();
    let full_words = needle_bits / WORD_BITS;
    let rem_bits = needle_bits % WORD_BITS;

    // Full u64 words — needle is always word-aligned at index 0 so we
    // compare needle_words[i] directly.
    for i in 0..full_words {
        let h = haystack.words.read_word_at(offset + i * WORD_BITS);
        if h != needle_words[i] {
            return false;
        }
    }

    // Last partial word (if any).
    if rem_bits > 0 {
        let mask = low_mask(rem_bits);
        let h = haystack.words.read_word_at(offset + full_words * WORD_BITS);
        if (h & mask) != (needle_words[full_words] & mask) {
            return false;
        }
    }

    true
}

mod impls_for_find;
mod impls_for_matches_at;
mod impls_for_strip;

#[cfg(test)]
mod tests_for_bits_equal_at;
