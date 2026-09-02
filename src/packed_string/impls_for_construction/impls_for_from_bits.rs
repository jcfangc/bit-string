use super::*;

impl<C, const BITS: u8> PackedString<C, BITS>
where
    C: PackedChar<BITS>,
{
    /// Validates and adopts an already packed bit payload.
    pub fn from_bits(bits: BitString) -> Option<Self> {
        let () = Self::VALID_WIDTH;
        let bits_per_char = usize::from(BITS);
        if bits.bit_len() % bits_per_char != 0 {
            return None;
        }
        let code_mask = u64::from(code_mask::<BITS>());
        for start in (0..bits.bit_len()).step_by(bits_per_char) {
            C::from_code((bits.get_chunk(start) & code_mask) as u8)?;
        }
        Some(Self {
            bits,
            marker: PhantomData,
        })
    }

    pub fn into_bits(self) -> BitString {
        unimplemented!("PackedString::into_bits")
    }
}
