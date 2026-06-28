//! Verify SIMD `cmp_aligned_words` against scalar for random inputs.

use proptest::prelude::*;

use crate::BitString;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 512,
        max_shrink_iters: 128,
        ..ProptestConfig::default()
    }
}

/// Scalar oracle: compare two bit strings bit by bit (LSB-first).
fn scalar_cmp(a: &BitString, b: &BitString) -> core::cmp::Ordering {
    use core::cmp::Ordering;
    let common = a.bit_len().min(b.bit_len());
    for i in 0..common {
        match (a.get(i), b.get(i)) {
            (Some(false), Some(true)) => return Ordering::Less,
            (Some(true), Some(false)) => return Ordering::Greater,
            _ => {}
        }
    }
    a.bit_len().cmp(&b.bit_len())
}

proptest! {
    #![proptest_config(config())]

    /// Small inputs — purely scalar path.
    #[test]
    fn small_inputs_match_scalar(
        a_bits in proptest::collection::vec(proptest::bool::ANY, 0..=64),
        b_bits in proptest::collection::vec(proptest::bool::ANY, 0..=64),
    ) {
        let a = BitString::from_bool_iter(a_bits);
        let b = BitString::from_bool_iter(b_bits);
        let result = a.as_bit_str().cmp(&b.as_bit_str());
        let expected = scalar_cmp(&a, &b);
        assert_eq!(result, expected);
    }

    /// Medium inputs — crossing the SIMD threshold.
    #[test]
    fn medium_inputs_match_scalar(
        a_bits in proptest::collection::vec(proptest::bool::ANY, 64..=320),
        b_bits in proptest::collection::vec(proptest::bool::ANY, 64..=320),
    ) {
        let a = BitString::from_bool_iter(a_bits);
        let b = BitString::from_bool_iter(b_bits);
        let result = a.as_bit_str().cmp(&b.as_bit_str());
        let expected = scalar_cmp(&a, &b);
        assert_eq!(result, expected);
    }

    /// Large inputs — SIMD main loop (many 4-word chunks on AVX2).
    #[test]
    fn large_inputs_match_scalar(
        a_bits in proptest::collection::vec(proptest::bool::ANY, 256..=512),
        b_bits in proptest::collection::vec(proptest::bool::ANY, 256..=512),
    ) {
        let a = BitString::from_bool_iter(a_bits);
        let b = BitString::from_bool_iter(b_bits);
        let result = a.as_bit_str().cmp(&b.as_bit_str());
        let expected = scalar_cmp(&a, &b);
        assert_eq!(result, expected);
    }

    /// Offset (unaligned) views — exercises the scalar read_word_at paths.
    #[test]
    fn offset_views_match_scalar(
        src_a in proptest::collection::vec(proptest::bool::ANY, 1..=256),
        src_b in proptest::collection::vec(proptest::bool::ANY, 1..=256),
        skip_a in 0usize..=32,
        skip_b in 0usize..=32,
    ) {
        let base_a = BitString::from_bool_iter(src_a);
        let base_b = BitString::from_bool_iter(src_b);
        let a_len = base_a.bit_len().saturating_sub(skip_a);
        let b_len = base_b.bit_len().saturating_sub(skip_b);
        if a_len == 0 || b_len == 0 {
            return Ok(());
        }
        let a = base_a.as_bit_str().slice_from(skip_a).slice_until(skip_a + a_len);
        let b = base_b.as_bit_str().slice_from(skip_b).slice_until(skip_b + b_len);
        let result = a.cmp(&b);

        // Reconstruct expected via to_bit_string roundtrip
        let expected = scalar_cmp(&a.to_bit_string(), &b.to_bit_string());
        assert_eq!(result, expected);
    }
}
