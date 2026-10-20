use alloc::vec;

use super::WordsUnpack;
use crate::traits::{WordsPack, layout_block_len};

#[test]
fn unpack_matches_codes_for_all_widths_and_layout_tails() {
    fn check<const BITS: u8>() {
        let mask = crate::code_mask::<BITS>();
        let codes = (0usize..257)
            .map(|index| ((index.wrapping_mul(37) + 11) as u8) & mask)
            .collect::<alloc::vec::Vec<_>>();
        let block_len = layout_block_len::<BITS>();

        for requested_len in [
            0, 1, 15, 16, 31, 32, 63, 64, 65, 79, 80, 95, 96, 127, 128, 129, 257,
        ] {
            let code_len = requested_len / block_len * block_len;
            let word_len = code_len * usize::from(BITS) / crate::WORD_BITS;
            let mut words = vec![0; word_len];
            words.as_mut_slice().pack_codes::<BITS>(&codes[..code_len]);

            let mut actual = vec![u8::MAX; code_len];
            words.as_slice().unpack_codes::<BITS>(&mut actual);
            assert_eq!(actual, codes[..code_len], "BITS={BITS}, len={code_len}");
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
