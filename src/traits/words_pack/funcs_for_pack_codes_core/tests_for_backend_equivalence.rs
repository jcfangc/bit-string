use alloc::vec;
use alloc::vec::Vec;

use super::scalar;
use crate::traits::layout_block_len;

fn codes<const BITS: u8>(len: usize) -> Vec<u8> {
    let mask = crate::code_mask::<BITS>();
    (0..len)
        .map(|index| ((index.wrapping_mul(29) + 7) as u8) & mask)
        .collect()
}

fn run<const BITS: u8>(codes: &[u8]) -> Vec<u64> {
    let words = codes.len() * usize::from(BITS) / 64;
    let mut dst = vec![0; words];
    scalar::pack_codes::<BITS>(&mut dst, codes);
    dst
}

#[test]
fn scalar_packs_complete_layout_blocks_for_all_widths() {
    for bits in 1..=8 {
        match bits {
            1 => assert_width::<1>(),
            2 => assert_width::<2>(),
            3 => assert_width::<3>(),
            4 => assert_width::<4>(),
            5 => assert_width::<5>(),
            6 => assert_width::<6>(),
            7 => assert_width::<7>(),
            8 => assert_width::<8>(),
            _ => unreachable!(),
        }
    }
}

fn assert_width<const BITS: u8>() {
    let block_len = layout_block_len::<BITS>();
    for blocks in 0..=3 {
        let input = codes::<BITS>(blocks * block_len);
        let actual = run::<BITS>(&input);

        let mut expected = vec![0; actual.len()];
        let mut bit = 0;
        for &code in &input {
            let word = bit / 64;
            let offset = bit % 64;
            expected[word] |= u64::from(code) << offset;
            if offset + usize::from(BITS) > 64 {
                expected[word + 1] |= u64::from(code) >> (64 - offset);
            }
            bit += usize::from(BITS);
        }

        assert_eq!(actual, expected, "BITS={BITS}, blocks={blocks}");
    }
}
