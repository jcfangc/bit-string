use int_interval::UsizeCO;

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
        Self::from_bits(self.bits.drain_interval(interval))
            .expect("PackedString invariant violated")
    }

    pub fn drain_interval_assign(&mut self, interval: UsizeCO) {
        let result = self.drain_interval(interval);
        *self = result;
    }
}
