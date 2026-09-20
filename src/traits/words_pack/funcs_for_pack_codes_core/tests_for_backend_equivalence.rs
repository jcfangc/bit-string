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

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
mod tests_for_avx2 {
    use super::super::avx2;
    use super::*;

    #[test]
    fn bits_four_matches_scalar_across_simd_prefix_and_tail() {
        for code_len in [0, 16, 32, 48, 64, 96, 128] {
            for input in [
                vec![0; code_len],
                vec![0x0f; code_len],
                codes::<4>(code_len),
            ] {
                let expected = run::<4>(&input);
                let mut actual = vec![u64::MAX; expected.len()];

                // SAFETY: This test is compiled only when AVX2 is enabled,
                // and the input satisfies the WordsPack contract.
                unsafe { avx2::pack_codes::<4>(&mut actual, &input) };

                assert_eq!(actual, expected, "code_len={code_len}, input={input:?}");
            }
        }
    }

    #[test]
    fn bits_three_matches_scalar_across_simd_prefix_and_tail() {
        for code_len in [0, 64, 128, 192, 256] {
            for input in [
                vec![0; code_len],
                vec![0x07; code_len],
                codes::<3>(code_len),
            ] {
                let expected = run::<3>(&input);
                let mut actual = vec![u64::MAX; expected.len()];

                // SAFETY: This test is compiled only when AVX2 is enabled,
                // and the input satisfies the WordsPack contract.
                unsafe { avx2::pack_codes::<3>(&mut actual, &input) };

                assert_eq!(actual, expected, "code_len={code_len}, input={input:?}");
            }
        }
    }

    #[test]
    fn bits_five_matches_scalar_across_simd_prefix_and_tail() {
        for code_len in [0, 64, 128, 192, 256] {
            for input in [
                vec![0; code_len],
                vec![0x1f; code_len],
                codes::<5>(code_len),
            ] {
                let expected = run::<5>(&input);
                let mut actual = vec![u64::MAX; expected.len()];

                // SAFETY: This test is compiled only when AVX2 is enabled,
                // and the input satisfies the WordsPack contract.
                unsafe { avx2::pack_codes::<5>(&mut actual, &input) };

                assert_eq!(actual, expected, "code_len={code_len}, input={input:?}");
            }
        }
    }

    #[test]
    fn bits_one_matches_scalar_across_simd_prefix_and_tail() {
        for code_len in [0, 64, 128, 192, 256] {
            for input in [vec![0; code_len], vec![1; code_len], codes::<1>(code_len)] {
                let expected = run::<1>(&input);
                let mut actual = vec![u64::MAX; expected.len()];

                // SAFETY: This test is compiled only when AVX2 is enabled,
                // and the input satisfies the WordsPack contract.
                unsafe { avx2::pack_codes::<1>(&mut actual, &input) };

                assert_eq!(actual, expected, "code_len={code_len}, input={input:?}");
            }
        }
    }

    #[test]
    fn bits_six_matches_scalar_across_simd_prefix_and_tail() {
        for code_len in [0, 32, 64, 96, 128] {
            for input in [
                vec![0; code_len],
                vec![0x3f; code_len],
                codes::<6>(code_len),
            ] {
                let expected = run::<6>(&input);
                let mut actual = vec![u64::MAX; expected.len()];

                // SAFETY: This test is compiled only when AVX2 is enabled,
                // and the input satisfies the WordsPack contract.
                unsafe { avx2::pack_codes::<6>(&mut actual, &input) };

                assert_eq!(actual, expected, "code_len={code_len}, input={input:?}");
            }
        }
    }
}
