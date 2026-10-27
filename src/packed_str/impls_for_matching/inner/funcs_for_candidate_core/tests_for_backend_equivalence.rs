use alloc::vec::Vec;
use proptest::prelude::*;

use crate::{BitString, low_mask};

use super::{matching_candidates, matching_lanes, scalar};

fn bit_string_from_codes<const BITS: u8>(codes: &[u8]) -> BitString {
    BitString::from_bool_iter(
        codes
            .iter()
            .copied()
            .flat_map(|code| (0..BITS).map(move |bit| code & (1 << bit) != 0)),
    )
}

fn assert_candidate_batches_match_scalar<const BITS: u8>() {
    let width = usize::from(BITS);
    let code_mask = ((1_u16 << BITS) - 1) as u8;
    let codes: Vec<u8> = (0_usize..=512)
        .map(|index| (index.wrapping_mul(37) as u8) & code_mask)
        .collect();
    let source = bit_string_from_codes::<BITS>(&codes);
    let haystack = source.as_bit_str().slice_from(width);
    let needle_mask = low_mask(width);

    for first_char in 0..64 {
        let windows = core::array::from_fn(|lane| haystack.get_chunk((first_char + lane) * width));
        let needle_first_word = windows[2] & needle_mask;
        let _expected = scalar::matching_lanes(&windows, needle_first_word, needle_mask, 4);
        let actual =
            matching_candidates::<BITS>(haystack, first_char, needle_first_word, needle_mask);

        #[cfg(any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "avx2"
            ),
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse4.1",
                not(target_feature = "avx2")
            ),
            all(target_arch = "aarch64", target_feature = "neon")
        ))]
        assert_eq!(
            actual,
            Some(_expected),
            "BITS={BITS}, first_char={first_char}"
        );

        #[cfg(not(any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "avx2"
            ),
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse4.1",
                not(target_feature = "avx2")
            ),
            all(target_arch = "aarch64", target_feature = "neon")
        )))]
        assert_eq!(actual, None);
    }
}

fn assert_candidate_batch_falls_back_at_backing_end<const BITS: u8>() {
    let width = usize::from(BITS);
    let source = bit_string_from_codes::<BITS>(&[0; 4]);
    let haystack = source.as_bit_str();

    assert_eq!(
        matching_candidates::<BITS>(haystack, 0, 0, low_mask(width)),
        None,
        "BITS={BITS} must fall back when a four-lane read reaches past backing words",
    );
}

proptest! {
    #[test]
    fn selected_backend_matches_scalar(
        windows in prop::array::uniform4(any::<u64>()),
        needle_first_word in any::<u64>(),
        needle_first_word_mask in any::<u64>(),
    ) {
        let expected = scalar::matching_lanes(
            &windows,
            needle_first_word,
            needle_first_word_mask,
            4,
        );
        let actual = matching_lanes(
            &windows,
            needle_first_word,
            needle_first_word_mask,
            4,
        );
        prop_assert_eq!(actual, expected);
    }
}

#[test]
fn candidate_backends_match_scalar_across_widths_view_offsets_and_word_phases() {
    assert_candidate_batches_match_scalar::<1>();
    assert_candidate_batches_match_scalar::<2>();
    assert_candidate_batches_match_scalar::<3>();
    assert_candidate_batches_match_scalar::<4>();
    assert_candidate_batches_match_scalar::<5>();
    assert_candidate_batches_match_scalar::<6>();
    assert_candidate_batches_match_scalar::<7>();
    assert_candidate_batches_match_scalar::<8>();
}

#[test]
fn candidate_backends_fall_back_when_the_backing_slice_lacks_prefetch_words() {
    assert_candidate_batch_falls_back_at_backing_end::<1>();
    assert_candidate_batch_falls_back_at_backing_end::<2>();
    assert_candidate_batch_falls_back_at_backing_end::<3>();
    assert_candidate_batch_falls_back_at_backing_end::<4>();
    assert_candidate_batch_falls_back_at_backing_end::<5>();
    assert_candidate_batch_falls_back_at_backing_end::<6>();
    assert_candidate_batch_falls_back_at_backing_end::<7>();
    assert_candidate_batch_falls_back_at_backing_end::<8>();
}
