use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn reverse(&self) -> Self {
        if matches!(BITS, 1 | 2 | 4 | 8) {
            return self.reverse_aligned_codes();
        }

        Self::from_chars(self.iter().rev())
    }

    pub fn reverse_assign(&mut self) {
        *self = self.reverse();
    }

    fn reverse_aligned_codes(&self) -> Self {
        let char_len = self.char_len();
        let bit_len = self.bits.bit_len();
        let words = self.bits.words();
        let mut reversed_words = alloc::vec::Vec::with_capacity(words.len());
        let chars_per_word = 64 / usize::from(BITS);

        for output_start in (0..char_len).step_by(chars_per_word) {
            let source_end = char_len - output_start;
            let source_start = source_end.saturating_sub(chars_per_word);
            let source_count = source_end - source_start;
            let source_bit_start = source_start * usize::from(BITS);
            let source_word_index = source_bit_start / 64;
            let source_shift = source_bit_start % 64;
            let mut source_window = words[source_word_index] >> source_shift;

            if source_shift != 0 && source_word_index + 1 < words.len() {
                source_window |= words[source_word_index + 1] << (64 - source_shift);
            }

            let source_bits = source_count * usize::from(BITS);
            if source_bits < 64 {
                source_window &= (1u64 << source_bits) - 1;
            }

            let reversed_window = match BITS {
                1 => source_window.reverse_bits(),
                2 => {
                    let reversed = source_window.reverse_bits();
                    ((reversed & 0x5555_5555_5555_5555) << 1)
                        | ((reversed & 0xaaaa_aaaa_aaaa_aaaa) >> 1)
                }
                4 => {
                    let reversed = source_window.swap_bytes();
                    ((reversed & 0x0f0f_0f0f_0f0f_0f0f) << 4)
                        | ((reversed & 0xf0f0_f0f0_f0f0_f0f0) >> 4)
                }
                8 => source_window.swap_bytes(),
                _ => unreachable!(),
            };
            reversed_words.push(reversed_window >> (64 - source_bits));
        }

        let bits = BitString::from_owned_words(reversed_words, bit_len)
            .expect("reversed byte words retain the original bit length");
        Self {
            bits,
            marker: core::marker::PhantomData,
        }
    }
}
