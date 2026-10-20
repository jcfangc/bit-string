use super::WordsPack;
use super::funcs_for_pack_codes_core;

impl WordsPack for [u64] {
    #[inline]
    fn pack_codes<const BITS: u8>(&mut self, codes: &[u8]) {
        funcs_for_pack_codes_core::pack_codes::<BITS>(self, codes);
    }
}
