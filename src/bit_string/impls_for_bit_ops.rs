mod impls_for_and;
mod impls_for_count_ones;
mod impls_for_not;
mod impls_for_or;
mod impls_for_shl;
mod impls_for_shr;
mod impls_for_xor;

use crate::bit_string::errors::BitStringLenMismatch;

use super::*;

impl BitString {
    #[inline]
    fn require_same_len(&self, rhs: &Self) -> Result<(), BitStringLenMismatch> {
        if self.bit_len == rhs.bit_len {
            Ok(())
        } else {
            Err(BitStringLenMismatch {
                lhs_len: self.bit_len,
                rhs_len: rhs.bit_len,
            })
        }
    }
}
