use crate::{WORD_BITS, assert_valid_width};

/// Fixed-width code unpacking operations on `[u64]` backing storage.
///
/// The input must contain complete layout blocks and the output length must
/// equal the number of codes represented by those words. Every output code is
/// fully overwritten and contains only its low `BITS` bits.
pub(crate) trait WordsUnpack {
    fn unpack_codes<const BITS: u8>(&self, codes: &mut [u8]);
}

#[inline]
pub(crate) const fn layout_block_len<const BITS: u8>() -> usize {
    assert_valid_width::<BITS>();
    WORD_BITS / gcd(WORD_BITS, BITS as usize)
}

#[inline]
const fn gcd(mut lhs: usize, mut rhs: usize) -> usize {
    while rhs != 0 {
        let remainder = lhs % rhs;
        lhs = rhs;
        rhs = remainder;
    }
    lhs
}

pub(crate) mod funcs_for_unpack_codes_core;
mod impls_for_u64_slice;

#[cfg(test)]
mod tests_for_words_unpack {
    use super::WordsUnpack;
    use crate::traits::WordsPack;

    #[test]
    fn scalar_and_dispatched_unpack_match_codes_for_all_widths() {
        fn check<const BITS: u8>() {
            let mask = if BITS == 8 {
                u8::MAX
            } else {
                (1u16 << BITS) as u8 - 1
            };
            let codes = (0..257)
                .map(|index| ((index * 37 + 11) as u8) & mask)
                .collect::<alloc::vec::Vec<_>>();
            let block_len = super::layout_block_len::<BITS>();
            for len in [
                0, 1, 15, 16, 31, 32, 63, 64, 65, 79, 80, 95, 96, 127, 128, 129, 257,
            ] {
                let full_len = len / block_len * block_len;
                let mut words = alloc::vec![0; full_len * usize::from(BITS) / 64];
                words.as_mut_slice().pack_codes::<BITS>(&codes[..full_len]);

                let mut actual = alloc::vec![0; full_len];
                words.as_slice().unpack_codes::<BITS>(&mut actual);
                assert_eq!(actual, codes[..full_len]);
            }
        }

        check::<1>();
        check::<2>();
        check::<3>();
        check::<4>();
        check::<5>();
        check::<6>();
        check::<7>();
        check::<8>();
    }
}
