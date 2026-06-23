use int_interval::UsizeCO;

use crate::BitString;

/// An empty view always has zero ones and zeros.
#[test]
fn counts_empty_view() {
    let bits = BitString::try_from("10110").unwrap();
    // Create an empty view by slicing beyond the source length.
    let v = bits.as_bitstr().slice(UsizeCO::try_new(10, 20).unwrap());

    assert_eq!(v.count_ones(), 0);
    assert_eq!(v.count_zeros(), 0);
}

/// Word-aligned views take the SIMD fast path.
#[test]
fn word_aligned_fast_path() {
    let mut bits = BitString::zeros(130);
    for i in [0, 63, 64, 65, 127, 128, 129] {
        bits.set(i, true);
    }
    // Full view: bit_len=130, aligned, 7 ones.
    let v = bits.as_bitstr();
    assert_eq!(v.count_ones(), 7);
    assert_eq!(v.count_zeros(), 123);

    // Slice from word-aligned boundary (64..130), 6 ones.
    let v = bits.as_bitstr().slice(UsizeCO::try_new(64, 130).unwrap());
    assert_eq!(v.count_ones(), 5);
}

/// Unaligned start: the first word is partial.
#[test]
fn unaligned_start() {
    let mut bits = BitString::zeros(130);
    // Set only bit 1 and bit 66 (word 0 offset 1, word 1 offset 2).
    bits.set(1, true);
    bits.set(66, true);

    // View from bit 1 to bit 130 → len 129, unaligned start.
    let v = bits.as_bitstr().slice(UsizeCO::try_new(1, 130).unwrap());
    assert_eq!(v.count_ones(), 2);
    assert_eq!(v.count_zeros(), 127);
}

/// All-ones string of various lengths.
#[test]
fn all_ones_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::ones(len);
        let v = bits.as_bitstr();
        assert_eq!(v.count_ones(), len, "len={len}");
        assert_eq!(v.count_zeros(), 0, "len={len}");
    }
}

/// All-zeros string of various lengths.
#[test]
fn all_zeros_at_various_lengths() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::zeros(len);
        let v = bits.as_bitstr();
        assert_eq!(v.count_ones(), 0, "len={len}");
        assert_eq!(v.count_zeros(), len, "len={len}");
    }
}

/// Mixed bits from a string pattern.
#[test]
fn counts_mixed_bits_from_string() {
    let bits = BitString::try_from("1010011100").unwrap();
    let v = bits.as_bitstr();

    assert_eq!(v.count_ones(), 5);
    assert_eq!(v.count_zeros(), 5);
}

/// Unaligned subrange within a single word.
#[test]
fn unaligned_single_word() {
    let bits = BitString::try_from("11110000").unwrap();
    // bits: 1 1 1 1 0 0 0 0

    // View bits 1..7 → "111000"
    let v = bits.as_bitstr().slice(UsizeCO::try_new(1, 7).unwrap());
    assert_eq!(v.count_ones(), 3);
    assert_eq!(v.count_zeros(), 3);

    // View bits 2..6 → "1100"
    let v = bits.as_bitstr().slice(UsizeCO::try_new(2, 6).unwrap());
    assert_eq!(v.count_ones(), 2);
    assert_eq!(v.count_zeros(), 2);
}

/// Invariant: count_ones + count_zeros == bit_len for every view.
#[test]
fn invariant_ones_plus_zeros_equals_bit_len() {
    let mut bits = BitString::zeros(200);
    for i in (0..200).step_by(7) {
        bits.set(i, true);
    }
    let full = bits.as_bitstr();

    for start in [0, 1, 5, 63, 64, 65, 127, 128] {
        for len in [10, 63, 64, 65, 128, 129] {
            let end = (start + len).min(full.bit_len());
            // Skip degenerate cases where start == end (empty interval not
            // constructible via try_new).
            if start == end {
                continue;
            }
            let v = full.slice(UsizeCO::try_new(start, end).unwrap());
            assert_eq!(
                v.count_ones() + v.count_zeros(),
                v.bit_len(),
                "start={start} end={end}"
            );
        }
    }
}

/// Unaligned view spanning many words exercises the SIMD middle-word path.
#[test]
fn unaligned_many_words() {
    let mut bits = BitString::zeros(300);
    // Set bits at known positions across multiple words.
    for i in [1, 63, 64, 100, 127, 128, 129, 200, 255, 256, 299] {
        bits.set(i, true);
    }

    // View from bit 1 (unaligned) covering bits 1..300 → 299 bits length.
    // Bit 1 is included; all 11 bits are in range [1, 300).
    let v = bits.as_bitstr().slice(UsizeCO::try_new(1, 300).unwrap());
    assert_eq!(v.count_ones(), 11);
    assert_eq!(v.count_zeros(), 288);
}

/// Word-aligned view exactly one word long.
#[test]
fn aligned_one_word() {
    let mut bits = BitString::zeros(128);
    bits.set(0, true);
    bits.set(63, true);

    // View word 0 (bits 0..64)
    let v = bits.as_bitstr().slice(UsizeCO::try_new(0, 64).unwrap());
    assert_eq!(v.count_ones(), 2);
    assert_eq!(v.count_zeros(), 62);

    // View word 1 (bits 64..128)
    let v = bits.as_bitstr().slice(UsizeCO::try_new(64, 128).unwrap());
    assert_eq!(v.count_ones(), 0);
    assert_eq!(v.count_zeros(), 64);
}
