use core::cmp::Ordering;

use super::*;

impl<C, const BITS: u8> PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    pub(crate) fn cmp_codes(&self, other: &Self) -> Ordering {
        let mask = u64::from(crate::code_mask::<BITS>());
        let shared_len = self.char_len().min(other.char_len());
        for index in 0..shared_len {
            let start = index * usize::from(BITS);
            let lhs = self.bits.get_chunk(start) & mask;
            let rhs = other.bits.get_chunk(start) & mask;
            match lhs.cmp(&rhs) {
                Ordering::Equal => {}
                ordering => return ordering,
            }
        }
        self.char_len().cmp(&other.char_len())
    }
}

impl<C, const BITS: u8> PartialOrd for PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_codes(other))
    }
}

impl<C, const BITS: u8> Ord for PackedStr<'_, C, BITS>
where
    C: PackedChar<BITS>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_codes(other)
    }
}
