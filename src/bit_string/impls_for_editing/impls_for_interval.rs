use int_interval::UsizeCO;

use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn replace_interval(&mut self, interval: UsizeCO, replacement: &Self) {
        Bits::assert_interval_in_bounds(interval, self.bit_len);

        let start = interval.start();
        let end = interval.end_excl();
        let tail_len = self.bit_len - end;

        // Fast path: replacement has the same length — overwrite in-place.
        if replacement.bit_len == interval.len() {
            // Clear the destination interval first: Bits::copy uses |=
            // for unaligned copies, so the target must be zeroed to
            // achieve a proper overwrite.
            let dst_end = start + replacement.bit_len;
            let first = start / WORD_BITS;
            let last = dst_end.saturating_sub(1) / WORD_BITS;

            if first == last {
                let mask = Bits::low_mask(replacement.bit_len) << (start % WORD_BITS);
                self.words[first] &= !mask;
            } else {
                self.words[first] &= Bits::low_mask(start % WORD_BITS);
                for w in (first + 1)..last {
                    self.words[w] = 0;
                }
                let end_rem = dst_end % WORD_BITS;
                if end_rem != 0 {
                    self.words[last] &= !Bits::low_mask(end_rem);
                } else {
                    self.words[last] = 0;
                }
            }

            Bits::copy(
                &replacement.words,
                0,
                &mut self.words,
                start,
                replacement.bit_len,
            );
            return;
        }

        let new_len = start
            .checked_add(replacement.bit_len)
            .and_then(|len| len.checked_add(tail_len))
            .expect("bit string length overflow");

        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.words, 0, &mut bits, 0, start);
        Bits::copy(&replacement.words, 0, &mut bits, start, replacement.bit_len);
        Bits::copy(
            &self.words,
            end,
            &mut bits,
            start + replacement.bit_len,
            tail_len,
        );

        self.words = bits;
        self.bit_len = new_len;
    }

    pub fn drain_interval(&mut self, interval: UsizeCO) -> Self {
        Bits::assert_interval_in_bounds(interval, self.bit_len);

        let start = interval.start();
        let end = interval.end_excl();
        let removed_len = interval.len();
        let tail_len = self.bit_len - end;

        let mut removed_bits = Bits::zero_words(Bits::word_len(removed_len));
        Bits::copy(&self.words, start, &mut removed_bits, 0, removed_len);

        let new_len = self.bit_len - removed_len;
        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.words, 0, &mut bits, 0, start);
        Bits::copy(&self.words, end, &mut bits, start, tail_len);

        self.words = bits;
        self.bit_len = new_len;

        Self {
            words: removed_bits,
            bit_len: removed_len,
        }
    }
}

#[cfg(test)]
mod tests_for_replace_interval;

#[cfg(test)]
mod tests_for_drain_interval;
