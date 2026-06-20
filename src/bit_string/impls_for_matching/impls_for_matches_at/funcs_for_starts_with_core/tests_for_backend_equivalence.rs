//! Verify SIMD word equality against scalar for random inputs.

use proptest::prelude::*;

use crate::BitString;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 512,
        max_shrink_iters: 128,
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn starts_with_matches_scalar(
        h_len in 64usize..=256,
        p_len in 1usize..=256,
    ) {
        prop_assume!(p_len <= h_len);
        let haystack = BitString::from_bool_iter((0..h_len).map(|i| (i * 17 + 3) % 5 == 0));
        let prefix = BitString::from_bool_iter((0..p_len).map(|i| (i * 17 + 3) % 5 == 0));

        let result = haystack.starts_with(&prefix);

        let expected = (0..p_len).all(|j| haystack.get(j) == prefix.get(j));
        assert_eq!(result, expected);
    }

    #[test]
    fn starts_with_random_data(
        h_bits in proptest::collection::vec(proptest::bool::ANY, 64..=256),
        p_bits in proptest::collection::vec(proptest::bool::ANY, 1..=64),
    ) {
        let haystack = BitString::from_bool_iter(h_bits);
        let prefix = BitString::from_bool_iter(p_bits);

        let result = haystack.starts_with(&prefix);
        let expected = prefix.bit_len() <= haystack.bit_len()
            && (0..prefix.bit_len()).all(|j| haystack.get(j) == prefix.get(j));

        assert_eq!(result, expected);
    }
}
