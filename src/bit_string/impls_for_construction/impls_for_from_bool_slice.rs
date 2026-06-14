use super::*;

impl From<&[bool]> for BitString {
    #[inline]
    fn from(values: &[bool]) -> Self {
        // SAFETY:
        // - `bool` has layout/size/alignment 1, so `*const bool` → `*const u8`
        //   is a valid pointer cast.
        // - Valid bool values are 0x00 (false) or 0x01 (true).
        let src = values.as_ptr() as *const u8;
        let len = values.len();
        Self {
            bits: funcs_for_pack_bools_core::bool_core(src, len),
            len,
        }
    }
}

impl<const N: usize> From<[bool; N]> for BitString {
    #[inline]
    fn from(values: [bool; N]) -> Self {
        // SAFETY:
        // - `bool` has layout/size/alignment 1, so `*const bool` → `*const u8`
        //   is a valid pointer cast.
        // - Valid bool values are 0x00 (false) or 0x01 (true).
        let src = values.as_ptr() as *const u8;
        Self {
            bits: funcs_for_pack_bools_core::bool_core(src, N),
            len: N,
        }
    }
}

mod funcs_for_pack_bools_core;
