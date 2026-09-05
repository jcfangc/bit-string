use int_intervals::UsizeCO;

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn drain_interval(&self, interval: UsizeCO) -> Self {
        let bits = usize::from(BITS);
        let start = interval.start().min(self.char_len());
        let end = interval.end_excl().min(self.char_len()).max(start);
        if start == end {
            return self.clone();
        }
        let interval = unsafe { UsizeCO::new_unchecked(start * bits, end * bits) };
        Self::from_valid_bits(self.bits.drain_interval(interval))
    }

    pub fn drain_interval_assign(&mut self, interval: UsizeCO) {
        let bits = usize::from(BITS);
        let start = interval.start().min(self.char_len());
        let end = interval.end_excl().min(self.char_len()).max(start);
        if start == end {
            return;
        }
        let interval = unsafe { UsizeCO::new_unchecked(start * bits, end * bits) };
        self.bits.drain_interval_assign(interval);
    }
}
