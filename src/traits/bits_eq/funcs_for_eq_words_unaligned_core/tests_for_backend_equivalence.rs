//! Verify SIMD shifted-window equality against scalar for random inputs.

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
    fn ends_with_matches_scalar(
        h_len in 64usize..=256,
        s_len in 1usize..=256,
    ) {
        prop_assume!(s_len <= h_len);
        let haystack = BitString::from_bool_iter((0..h_len).map(|i| (i * 13 + 7) % 5 == 0));
        let suffix = BitString::from_bool_iter((0..s_len).map(|i| (i * 13 + 7) % 5 == 0));

        let result = haystack.ends_with(suffix.as_bit_str());
        let start = h_len - s_len;
        let expected = (0..s_len).all(|j| haystack.get(start + j) == suffix.get(j));
        assert_eq!(result, expected);
    }

    #[test]
    fn ends_with_random_data(
        h_bits in proptest::collection::vec(proptest::bool::ANY, 64..=256),
        s_bits in proptest::collection::vec(proptest::bool::ANY, 1..=64),
    ) {
        let haystack = BitString::from_bool_iter(h_bits);
        let suffix = BitString::from_bool_iter(s_bits);

        let result = haystack.ends_with(suffix.as_bit_str());
        let expected = suffix.bit_len() <= haystack.bit_len()
            && {
                let start = haystack.bit_len() - suffix.bit_len();
                (0..suffix.bit_len()).all(|j| haystack.get(start + j) == suffix.get(j))
            };

        assert_eq!(result, expected);
    }
}
