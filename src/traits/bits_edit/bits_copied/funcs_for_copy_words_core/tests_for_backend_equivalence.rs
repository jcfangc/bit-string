//! Verify copy_words against scalar for random inputs.

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

fn scalar_copy(dst: &mut [u64], src: &[u64], count: usize) {
    for i in 0..count {
        dst[i] = src[i];
    }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn copy_words_matches_scalar(count in 1usize..=256) {
        let extra = 16;
        let src: Vec<u64> = (0..count + extra)
            .map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
            .collect();
        let mut dst_impl = vec![0u64; count + extra];
        let mut dst_scalar = vec![0u64; count + extra];

        super::copy_words(&mut dst_impl, &src, count);
        scalar_copy(&mut dst_scalar, &src, count);

        assert_eq!(&dst_impl[..count], &dst_scalar[..count]);
    }
}
