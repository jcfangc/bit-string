use int_intervals::UsizeCO;

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn slice(&self, interval: UsizeCO) -> Self {
        let start = interval.start().min(self.char_len());
        let end = interval.end_excl().min(self.char_len()).max(start);
        if start == end {
            return Self::new();
        }
        let bits = usize::from(BITS);
        Self::from_valid_bits(
            self.bits.slice(
                UsizeCO::checked_from_start_len(start * bits, (end - start) * bits).unwrap(),
            ),
        )
    }

    pub fn slice_from(&self, start: usize) -> Self {
        if start >= self.char_len() {
            return Self::new();
        }
        self.slice(
            UsizeCO::checked_from_start_len(start, self.char_len().saturating_sub(start)).unwrap(),
        )
    }

    pub fn slice_until(&self, end: usize) -> Self {
        if end == 0 {
            return Self::new();
        }
        self.slice(UsizeCO::checked_from_start_len(0, end).unwrap())
    }
}
