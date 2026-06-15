use int_interval::UsizeCO;

use crate::bit_string::bits::Bits;

use super::*;

impl BitString {
    pub fn replace_interval(&mut self, interval: UsizeCO, replacement: &Self) {
        Bits::assert_interval_in_bounds(interval, self.len);

        let start = interval.start();
        let end = interval.end_excl();
        let tail_len = self.len - end;

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
