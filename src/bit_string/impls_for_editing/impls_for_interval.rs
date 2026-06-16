use int_interval::UsizeCO;

use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn replace_interval(&mut self, interval: UsizeCO, replacement: &Self) {
        Bits::assert_interval_in_bounds(interval, self.len);

        let start = interval.start();
        let end = interval.end_excl();
        let tail_len = self.len - end;

        // Fast path: replacement has the same length — overwrite in-place.
        if replacement.len == interval.len() {
            // Clear the destination interval first: Bits::copy uses |=
            // for unaligned copies, so the target must be zeroed to
            // achieve a proper overwrite.
            let dst_end = start + replacement.len;
            let first = start / WORD_BITS;
            let last = dst_end.saturating_sub(1) / WORD_BITS;

            if first == last {
                let mask = Bits::low_mask(replacement.len) << (start % WORD_BITS);
                self.bits[first] &= !mask;
            } else {
                self.bits[first] &= Bits::low_mask(start % WORD_BITS);
                for w in (first + 1)..last {
                    self.bits[w] = 0;
                }
                let end_rem = dst_end % WORD_BITS;
                if end_rem != 0 {
                    self.bits[last] &= !Bits::low_mask(end_rem);
                } else {
                    self.bits[last] = 0;
                }
            }

            Bits::copy(&replacement.bits, 0, &mut self.bits, start, replacement.len);
            return;
        }

        let new_len = start
            .checked_add(replacement.len)
            .and_then(|len| len.checked_add(tail_len))
            .expect("bit string length overflow");

        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, start);
        Bits::copy(&replacement.bits, 0, &mut bits, start, replacement.len);
        Bits::copy(
            &self.bits,
            end,
            &mut bits,
            start + replacement.len,
            tail_len,
        );

        self.bits = bits;
        self.len = new_len;
    }

    pub fn drain_interval(&mut self, interval: UsizeCO) -> Self {
        Bits::assert_interval_in_bounds(interval, self.len);

        let start = interval.start();
        let end = interval.end_excl();
        let removed_len = interval.len();
        let tail_len = self.len - end;

        let mut removed_bits = Bits::zero_words(Bits::word_len(removed_len));
        Bits::copy(&self.bits, start, &mut removed_bits, 0, removed_len);

        let new_len = self.len - removed_len;
        let mut bits = Bits::zero_words(Bits::word_len(new_len));

        Bits::copy(&self.bits, 0, &mut bits, 0, start);
        Bits::copy(&self.bits, end, &mut bits, start, tail_len);

        self.bits = bits;
        self.len = new_len;

        Self {
            bits: removed_bits,
            len: removed_len,
        }
    }
}

#[cfg(test)]
mod tests_for_replace_interval;

#[cfg(test)]
mod tests_for_drain_interval;
