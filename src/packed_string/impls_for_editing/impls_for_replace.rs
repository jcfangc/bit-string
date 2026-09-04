use int_interval::UsizeCO;

use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    pub fn replace_interval(&self, interval: UsizeCO, replacement: &Self) -> Self {
        let start = interval.start().min(self.char_len());
        let end = interval.end_excl().min(self.char_len()).max(start);
        self.replace_range(start, end, replacement)
    }

    pub fn replace_interval_assign(&mut self, interval: UsizeCO, replacement: &Self) {
        let result = self.replace_interval(interval, replacement);
        *self = result;
    }

    pub fn replace(&self, start: usize, replacement: &Self) -> Self {
        let start = start.min(self.char_len());
        let end = start
            .saturating_add(replacement.char_len())
            .min(self.char_len());
        self.replace_range(start, end, replacement)
    }

    pub fn replace_assign(&mut self, start: usize, replacement: &Self) {
        let result = self.replace(start, replacement);
        *self = result;
    }

    fn replace_range(&self, start: usize, end: usize, replacement: &Self) -> Self {
        let bits = usize::from(BITS);
        let result = self.bits.replace(start * bits, &replacement.bits);
        Self::from_bits(if end > start {
            let interval = unsafe { UsizeCO::new_unchecked(start * bits, end * bits) };
            self.bits.replace_interval(interval, &replacement.bits)
        } else {
            result
        })
        .expect("PackedString invariant violated")
    }
}
