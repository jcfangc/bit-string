use int_interval::UsizeCO;

use super::*;

impl<'ps, C, const BITS: u8> PackedStr<'ps, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn slice(&self, interval: UsizeCO) -> Self {
        let bits = usize::from(BITS);
        let start = interval.start().min(self.char_len()).saturating_mul(bits);
        let end = interval
            .end_excl()
            .min(self.char_len())
            .saturating_mul(bits);
        if end <= start {
            return Self::from_valid_bit_str(self.bits.slice_until(0));
        }
        let interval = UsizeCO::checked_from_start_len(start, end - start).unwrap();
        Self::from_valid_bit_str(self.bits.slice(interval))
    }

    pub fn slice_from(&self, start: usize) -> Self {
        if start >= self.char_len() {
            return Self::from_valid_bit_str(self.bits.slice_from(self.bits.bit_len()));
        }
        self.slice(UsizeCO::checked_from_start_len(start, self.char_len() - start).unwrap())
    }

    pub fn slice_until(&self, end: usize) -> Self {
        if end == 0 {
            return Self::from_valid_bit_str(self.bits.slice_until(0));
        }
        self.slice(UsizeCO::checked_from_start_len(0, end).unwrap())
    }
}
