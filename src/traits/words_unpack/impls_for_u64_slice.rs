use super::WordsUnpack;
use super::funcs_for_unpack_codes_core;

impl WordsUnpack for [u64] {
    #[inline]
    fn unpack_codes<const BITS: u8>(&self, codes: &mut [u8]) {
        funcs_for_unpack_codes_core::unpack_codes::<BITS>(self, codes);
    }
}
