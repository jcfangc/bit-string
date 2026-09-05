//! Verify SIMD copy_words_shifted against scalar for random inputs.

use alloc::vec;
use alloc::vec::Vec;
use proptest::prelude::*;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        max_shrink_iters: 64,
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn copy_words_shifted_matches_scalar(
        count in 1usize..=256,
        shift in 1usize..=63,
    ) {
        let extra = 16; // padding so src has count+1 valid words
        let src: Vec<u64> = (0..count + 1 + extra).map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)).collect();
        let mut dst_simd = vec![0u64; count + extra];
        let mut dst_scalar = vec![0u64; count + extra];

        super::copy_words_shifted(&mut dst_simd, &src, count, shift);
        super::scalar::copy_words_shifted(&mut dst_scalar, &src, count, shift);

        assert_eq!(&dst_simd[..count], &dst_scalar[..count]);
    }
}
