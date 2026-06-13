use alloc::vec;
use alloc::vec::Vec;

use super::Bits;
use crate::WORD_BITS;

fn words_from_indices(indices: &[usize], word_count: usize) -> Vec<u64> {
    let mut words = vec![0; word_count];

    for &index in indices {
        Bits::set_bit(&mut words, index, true);
    }

    words
}

fn collect_bits(words: &[u64], len: usize) -> Vec<bool> {
    (0..len).map(|index| Bits::bit_at(words, index)).collect()
}

#[test]
fn copies_zero_bits_without_changing_destination() {
    let src = words_from_indices(&[0, 3, 65], 2);
    let mut dst = words_from_indices(&[1, 4, 70], 2);
    let before = dst.clone();

    Bits::copy(&src, 0, &mut dst, 10, 0);

    assert_eq!(dst, before);
}

#[test]
fn copies_aligned_full_word_into_empty_destination() {
    let src = [0x0123_4567_89ab_cdef];
    let mut dst = [0];

    Bits::copy(&src, 0, &mut dst, 0, WORD_BITS);

    assert_eq!(dst, src);
}

#[test]
fn copies_partial_bits_from_unaligned_source_to_aligned_destination() {
    let src = words_from_indices(&[3, 5, 8, 13], 1);
    let mut dst = [0];

    Bits::copy(&src, 3, &mut dst, 0, 6);

    assert_eq!(
        collect_bits(&dst, 6),
        vec![true, false, true, false, false, true]
    );
}

#[test]
fn copies_partial_bits_from_aligned_source_to_unaligned_destination() {
    let src = words_from_indices(&[0, 2, 5], 1);
    let mut dst = [0];

    Bits::copy(&src, 0, &mut dst, 4, 6);

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

    Bits::copy(&src, WORD_BITS - 2, &mut dst, 0, 6);

    assert_eq!(
        collect_bits(&dst, 6),
        vec![true, false, true, false, false, true]
    );
}

#[test]
fn copies_across_destination_word_boundary() {
    let src = words_from_indices(&[0, 2, 5], 1);
    let mut dst = [0, 0];

    Bits::copy(&src, 0, &mut dst, WORD_BITS - 2, 6);

    assert_eq!(Bits::bit_at(&dst, WORD_BITS - 2), true);
    assert_eq!(Bits::bit_at(&dst, WORD_BITS - 1), false);
    assert_eq!(Bits::bit_at(&dst, WORD_BITS), true);
    assert_eq!(Bits::bit_at(&dst, WORD_BITS + 1), false);
    assert_eq!(Bits::bit_at(&dst, WORD_BITS + 2), false);
    assert_eq!(Bits::bit_at(&dst, WORD_BITS + 3), true);
}

#[test]
fn leaves_bits_outside_destination_range_unchanged() {
    let src = words_from_indices(&[0, 2, 5], 1);
    let mut dst = words_from_indices(&[1, 20], 1);

    Bits::copy(&src, 0, &mut dst, 4, 6);

    assert_eq!(Bits::bit_at(&dst, 1), true);
    assert_eq!(Bits::bit_at(&dst, 20), true);

    assert_eq!(Bits::bit_at(&dst, 4), true);
    assert_eq!(Bits::bit_at(&dst, 5), false);
    assert_eq!(Bits::bit_at(&dst, 6), true);
    assert_eq!(Bits::bit_at(&dst, 7), false);
    assert_eq!(Bits::bit_at(&dst, 8), false);
    assert_eq!(Bits::bit_at(&dst, 9), true);
}

#[test]
fn copies_more_than_one_chunk() {
    let src = words_from_indices(&[0, 63, 64, 70, 127, 128, 129], 3);
    let mut dst = vec![0; 3];

    Bits::copy(&src, 0, &mut dst, 0, WORD_BITS * 2 + 2);

    for index in 0..(WORD_BITS * 2 + 2) {
        assert_eq!(
            Bits::bit_at(&dst, index),
            Bits::bit_at(&src, index),
            "index={index}"
        );
    }
}
