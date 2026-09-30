#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use super::scalar;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use crate::traits::WordsPack;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use alloc::vec;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
#[test]
fn avx2_bits_four_matches_scalar_across_prefix_and_tail() {
    fn codes<const BITS: u8>(len: usize) -> alloc::vec::Vec<u8> {
        let mask = crate::code_mask::<BITS>();
        (0..len)
            .map(|index| ((index.wrapping_mul(29) + 7) as u8) & mask)
            .collect()
    }

    for code_len in [0, 16, 32, 48, 64, 80, 96, 112, 128, 144] {
        let input = codes::<4>(code_len);
        let words_len = code_len / 16;
        let mut words = vec![0; words_len];
        words.as_mut_slice().pack_codes::<4>(&input);

        let mut expected = vec![0; code_len];
        scalar::unpack_codes::<4>(&words, &mut expected);

        let mut actual = vec![u8::MAX; code_len];
        // SAFETY: This test only compiles when AVX2 is enabled, and the packed
        // words and output satisfy the complete-layout-block contract.
        unsafe { super::avx2::unpack_codes::<4>(&words, &mut actual) };

        assert_eq!(actual, expected, "code_len={code_len}");
    }
}
