//! Verify SIMD unaligned word comparison against scalar for random inputs.

use core::cmp::Ordering;

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
fn scalar_cmp_bits(a: &BitString, b: &BitString) -> Ordering {
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

    /// Haystack unaligned, needle aligned — exercises `cmp_unaligned_words`.
    #[test]
    fn unaligned_hs_vs_aligned_nd(
        hs_bits in proptest::collection::vec(proptest::bool::ANY, 65..=320),
        nd_bits in proptest::collection::vec(proptest::bool::ANY, 1..=256),
        skip in 1usize..=32,
    ) {
        let base = BitString::from_bool_iter(hs_bits);
        let hs_len = base.bit_len().saturating_sub(skip);
        if hs_len == 0 || hs_len < nd_bits.len() {
            return Ok(());
        }
        let hs = base.as_bit_str().slice_from(skip).slice_until(skip + hs_len);
        let nd = BitString::from_bool_iter(nd_bits);

        let result = hs.cmp(&nd.as_bit_str());
        let expected = scalar_cmp_bits(&hs.to_bit_string(), &nd);
        assert_eq!(result, expected);
    }

    /// Both unaligned — exercises the scalar fallback (double read_word_at).
    #[test]
    fn both_unaligned(
        src_a in proptest::collection::vec(proptest::bool::ANY, 65..=256),
        src_b in proptest::collection::vec(proptest::bool::ANY, 65..=256),
        skip_a in 1usize..=32,
        skip_b in 1usize..=32,
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
        let expected = scalar_cmp_bits(&a.to_bit_string(), &b.to_bit_string());
        assert_eq!(result, expected);
    }

    /// Long unaligned haystack vs aligned needle — SIMD main loop.
    #[test]
    fn long_unaligned_vs_aligned(
        hs_bits in proptest::collection::vec(proptest::bool::ANY, 256..=512),
        nd_bits in proptest::collection::vec(proptest::bool::ANY, 64..=320),
        skip in 1usize..=32,
    ) {
        let base = BitString::from_bool_iter(hs_bits);
        let hs_len = base.bit_len().saturating_sub(skip);
        if hs_len < nd_bits.len() {
            return Ok(());
        }
        let hs = base.as_bit_str().slice_from(skip).slice_until(skip + hs_len);
        let nd = BitString::from_bool_iter(nd_bits);

        let result = hs.cmp(&nd.as_bit_str());
        let expected = scalar_cmp_bits(&hs.to_bit_string(), &nd);
        assert_eq!(result, expected);
    }
}
