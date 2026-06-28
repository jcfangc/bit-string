use alloc::vec;
use alloc::vec::Vec;

use super::*;
use crate::WORD_BITS;

fn words_from_indices(indices: &[usize], word_count: usize) -> Vec<u64> {
    let mut words = vec![0; word_count];

    for &index in indices {
        words.set_bit_at(index, true);
    }

    words
}

fn collect_bits(words: &[u64], len: usize) -> Vec<bool> {
    (0..len).map(|index| words.read_bit_at(index)).collect()
}

#[test]
fn copies_zero_bits_without_changing_destination() {
    let src = words_from_indices(&[0, 3, 65], 2);
    let mut dst = words_from_indices(&[1, 4, 70], 2);
    let before = dst.clone();

    src.copy_bits(0, 0).paste_to(&mut dst, 10);

    assert_eq!(dst, before);
}

#[test]
fn copies_aligned_full_word_into_empty_destination() {
    let src = [0x0123_4567_89ab_cdef];
    let mut dst = [0];

    src.copy_bits(0, WORD_BITS).paste_to(&mut dst, 0);

    assert_eq!(dst, src);
}

#[test]
fn copies_partial_bits_from_unaligned_source_to_aligned_destination() {
    let src = words_from_indices(&[3, 5, 8, 13], 1);
    let mut dst = [0];

    src.copy_bits(3, 6).paste_to(&mut dst, 0);

    assert_eq!(
        collect_bits(&dst, 6),
        vec![true, false, true, false, false, true]
    );
}

#[test]
fn copies_partial_bits_from_aligned_source_to_unaligned_destination() {
    let src = words_from_indices(&[0, 2, 5], 1);
    let mut dst = [0];

    src.copy_bits(0, 6).paste_to(&mut dst, 4);

    assert_eq!(collect_bits(&dst, 4), vec![false, false, false, false]);
    assert_eq!(
        collect_bits(&dst, 10)[4..],
        [true, false, true, false, false, true]
    );
}

#[test]
fn copies_across_source_word_boundary() {
    let src = words_from_indices(&[WORD_BITS - 2, WORD_BITS, WORD_BITS + 3], 2);
    let mut dst = [0];

    src.copy_bits(WORD_BITS - 2, 6).paste_to(&mut dst, 0);

    assert_eq!(
        collect_bits(&dst, 6),
        vec![true, false, true, false, false, true]
    );
}

#[test]
fn copies_across_destination_word_boundary() {
    let src = words_from_indices(&[0, 2, 5], 1);
    let mut dst = [0, 0];

    src.copy_bits(0, 6).paste_to(&mut dst, WORD_BITS - 2);

    assert_eq!(dst.read_bit_at(WORD_BITS - 2), true);
    assert_eq!(dst.read_bit_at(WORD_BITS - 1), false);
    assert_eq!(dst.read_bit_at(WORD_BITS), true);
    assert_eq!(dst.read_bit_at(WORD_BITS + 1), false);
    assert_eq!(dst.read_bit_at(WORD_BITS + 2), false);
    assert_eq!(dst.read_bit_at(WORD_BITS + 3), true);
}

#[test]
fn leaves_bits_outside_destination_range_unchanged() {
    let src = words_from_indices(&[0, 2, 5], 1);
    let mut dst = words_from_indices(&[1, 20], 1);

    src.copy_bits(0, 6).paste_to(&mut dst, 4);

    assert_eq!(dst.read_bit_at(1), true);
    assert_eq!(dst.read_bit_at(20), true);

    assert_eq!(dst.read_bit_at(4), true);
    assert_eq!(dst.read_bit_at(5), false);
    assert_eq!(dst.read_bit_at(6), true);
    assert_eq!(dst.read_bit_at(7), false);
    assert_eq!(dst.read_bit_at(8), false);
    assert_eq!(dst.read_bit_at(9), true);
}

#[test]
fn copies_more_than_one_chunk() {
    let src = words_from_indices(&[0, 63, 64, 70, 127, 128, 129], 3);
    let mut dst = vec![0; 3];

    src.copy_bits(0, WORD_BITS * 2 + 2).paste_to(&mut dst, 0);

    for index in 0..(WORD_BITS * 2 + 2) {
        assert_eq!(
            dst.read_bit_at(index),
            src.read_bit_at(index),
            "index={index}"
        );
    }
}

// ---------------------------------------------------------------------------
// Multi-word SIMD path tests — each case needs enough full_words to hit the
// SIMD threshold (SMALL_WORDS on AVX2 = 4, SSE2/NEON = 2).
// ---------------------------------------------------------------------------

/// Helper: scalar copy via read_word_at + write_word_at, used as oracle.
fn scalar_paste(src: &[u64], src_start: usize, len: usize, dst: &mut [u64], dst_start: usize) {
    let full_words = len / WORD_BITS;
    let rem = len % WORD_BITS;
    for i in 0..full_words {
        let chunk = src.read_word_at::<false>(src_start + i * WORD_BITS);
        dst.write_word_at::<false>(dst_start + i * WORD_BITS, chunk, WORD_BITS);
    }
    if rem > 0 {
        let chunk = src.read_word_at::<false>(src_start + full_words * WORD_BITS);
        dst.write_word_at::<false>(dst_start + full_words * WORD_BITS, chunk, rem);
    }
}

#[test]
fn case2_unaligned_src_multiword_vs_scalar() {
    // Case 2: src unaligned, dst aligned — exercises copy_words_shifted.
    for count in [4, 16, 100] {
        let src: Vec<u64> = (0..count + 2)
            .map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
            .collect();
        let len = count * WORD_BITS;
        let src_start = 21; // unaligned
        let dst_start = 0; // aligned

        let mut dst_impl = vec![0; count + 1];
        let mut dst_oracle = vec![0; count + 1];

        src.copy_bits(src_start, len)
            .paste_to(&mut dst_impl, dst_start);
        scalar_paste(&src, src_start, len, &mut dst_oracle, dst_start);

        assert_eq!(&dst_impl[..count], &dst_oracle[..count], "count={count}");
    }
}

#[test]
fn case3_aligned_src_unaligned_dst_multiword_vs_scalar() {
    // Case 3: src aligned, dst unaligned — exercises reversed shift SIMD.
    for count in [4, 16, 100] {
        let src: Vec<u64> = (0..count + 2)
            .map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
            .collect();
        let len = count * WORD_BITS;
        let src_start = 0; // aligned
        let dst_start = 21; // unaligned

        let mut dst_impl = vec![0; count + 2];
        let mut dst_oracle = vec![0; count + 2];

        src.copy_bits(src_start, len)
            .paste_to(&mut dst_impl, dst_start);
        scalar_paste(&src, src_start, len, &mut dst_oracle, dst_start);

        assert_eq!(
            &dst_impl[..count + 2],
            &dst_oracle[..count + 2],
            "count={count}"
        );
    }
}

#[test]
fn case3_preserves_existing_dst_bits_below_dst_shift() {
    // The first boundary word ORs into dst — bits below dst_shift must survive.
    let src = words_from_indices(&[0, 10, 20, 30, 40, 50], 2);
    let existing_bit = 3; // below dst_shift = 5
    let mut dst = vec![0; 3];
    dst.set_bit_at(existing_bit, true);

    // This hits Case 3: src_start = 0 (aligned), dst_start = 5 (unaligned).
    src.copy_bits(0, 128).paste_to(&mut dst, 5);

    // The existing bit at position 3 must still be set.
    assert!(
        dst.read_bit_at(existing_bit),
        "existing bit below dst_shift was clobbered"
    );
}

#[test]
fn case4_both_unaligned_multiword_vs_scalar() {
    // Case 4: both src and dst unaligned — scalar fallback.
    for count in [4, 16] {
        let src: Vec<u64> = (0..count + 3)
            .map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
            .collect();
        let len = count * WORD_BITS;
        let src_start = 13; // unaligned
        let dst_start = 37; // unaligned

        let mut dst_impl = vec![0; count + 3];
        let mut dst_oracle = vec![0; count + 3];

        src.copy_bits(src_start, len)
            .paste_to(&mut dst_impl, dst_start);
        scalar_paste(&src, src_start, len, &mut dst_oracle, dst_start);

        assert_eq!(
            &dst_impl[..count + 3],
            &dst_oracle[..count + 3],
            "count={count}"
        );
    }
}

// ---------------------------------------------------------------------------
// Adversarial tests — edge cases targeting paste_to boundary logic.
// ---------------------------------------------------------------------------

#[test]
fn case3_guard_exactly_one_extra_word_hits_simd() {
    // src.len() == sw + full_words + 1 — exactly enough for the SIMD spill read.
    let count = 16; // full_words = 16, mid_count = 15
    let src: Vec<u64> = (0..count + 1) // sw=0, src.len() = count+1 = sw+full_words+1
        .map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
        .collect();
    let len = count * WORD_BITS;
    let mut dst_simd = vec![0; count + 2];
    let mut dst_oracle = vec![0; count + 2];

    src.copy_bits(0, len).paste_to(&mut dst_simd, 21);
    scalar_paste(&src, 0, len, &mut dst_oracle, 21);

    assert_eq!(&dst_simd[..count + 2], &dst_oracle[..count + 2]);
}

#[test]
fn case3_guard_exact_fit_falls_to_scalar() {
    // src.len() == sw + full_words — no spare word for SIMD spill, falls to Case 4.
    let count = 16;
    let src: Vec<u64> = (0..count) // sw=0, src.len() = count = sw+full_words
        .map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
        .collect();
    let len = count * WORD_BITS;
    let mut dst_impl = vec![0; count + 2];
    let mut dst_oracle = vec![0; count + 2];

    src.copy_bits(0, len).paste_to(&mut dst_impl, 21);
    scalar_paste(&src, 0, len, &mut dst_oracle, 21);

    assert_eq!(&dst_impl[..count + 2], &dst_oracle[..count + 2]);
}

#[test]
fn case3_remainder_one_bit_at_extreme_shifts() {
    // rem=1 + dst_shift ∈ {1, 63} — stress the overlap between high boundary
    // OR and remainder OR into the same dst word.
    for dst_shift in [1, 63] {
        let count = 16;
        let rem = 1;
        let len = count * WORD_BITS + rem;
        let src: Vec<u64> = (0..count + 2)
            .map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
            .collect();
        let mut dst_impl = vec![0; count + 3];
        let mut dst_oracle = vec![0; count + 3];

        src.copy_bits(0, len).paste_to(&mut dst_impl, dst_shift);
        scalar_paste(&src, 0, len, &mut dst_oracle, dst_shift);

        assert_eq!(
            &dst_impl[..count + 3],
            &dst_oracle[..count + 3],
            "dst_shift={dst_shift}, rem=1"
        );
    }
}

#[test]
fn case3_remainder_63bits_at_extreme_shifts() {
    // rem=63 + dst_shift ∈ {1, 63} — nearly full word remainder.
    for dst_shift in [1, 63] {
        let count = 16;
        let rem = 63;
        let len = count * WORD_BITS + rem;
        let src: Vec<u64> = (0..count + 2)
            .map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
            .collect();
        let mut dst_impl = vec![0; count + 3];
        let mut dst_oracle = vec![0; count + 3];

        src.copy_bits(0, len).paste_to(&mut dst_impl, dst_shift);
        scalar_paste(&src, 0, len, &mut dst_oracle, dst_shift);

        assert_eq!(
            &dst_impl[..count + 3],
            &dst_oracle[..count + 3],
            "dst_shift={dst_shift}, rem=63"
        );
    }
}

#[test]
fn case2_remainder_1bit_writes_correct_word_boundary() {
    // rem=1 — single bit chunk, write_word_at should not corrupt spill.
    for src_shift in [1, 63] {
        let count = 16;
        let rem = 1;
        let len = count * WORD_BITS + rem;
        let src: Vec<u64> = (0..count + 2)
            .map(|i| (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
            .collect();
        let mut dst_impl = vec![0; count + 2];
        let mut dst_oracle = vec![0; count + 2];

        src.copy_bits(src_shift, len).paste_to(&mut dst_impl, 0);
        scalar_paste(&src, src_shift, len, &mut dst_oracle, 0);

        assert_eq!(
            &dst_impl[..count + 2],
            &dst_oracle[..count + 2],
            "src_shift={src_shift}, rem=1"
        );
    }
}
