use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn reverse(&self) -> Self {
        if BITS == 8 {
            return self.reverse_bytes();
        }

        Self::from_chars(self.iter().rev())
    }

    pub fn reverse_assign(&mut self) {
        *self = self.reverse();
    }

    fn reverse_bytes(&self) -> Self {
        let char_len = self.char_len();
        let bit_len = self.bits.bit_len();
        let words = self.bits.words();
        let mut reversed_words = alloc::vec::Vec::with_capacity(words.len());

        for output_start in (0..char_len).step_by(8) {
            let source_end = char_len - output_start;
            let source_start = source_end.saturating_sub(8);
            let source_count = source_end - source_start;
            let source_bit_start = source_start * 8;
            let source_word_index = source_bit_start / 64;
            let source_shift = source_bit_start % 64;
            let mut source_window = words[source_word_index] >> source_shift;

            if source_shift != 0 && source_word_index + 1 < words.len() {
                source_window |= words[source_word_index + 1] << (64 - source_shift);
            }

            let source_bits = source_count * 8;
            if source_bits < 64 {
                source_window &= (1u64 << source_bits) - 1;
            }

            reversed_words.push(source_window.swap_bytes() >> ((8 - source_count) * 8));
        }

        let bits = BitString::from_owned_words(reversed_words, bit_len)
            .expect("reversed byte words retain the original bit length");
        Self {
            bits,
            marker: core::marker::PhantomData,
        }
    }
}
