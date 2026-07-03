use super::*;
use int_interval::UsizeCO;

// ===========================================================================
// A. BitString leading/trailing — exhaustive small-length coverage
// ===========================================================================

#[test]
fn attack_bitstring_leading_trailing_exhaustive() {
    // Exhaustively test every meaningful length from 0 to 256 with a
    // family of patterns that stress every code path.
    for len in 0..=256 {
        let z = BitString::zeros(len);
        assert_eq!(z.leading_zeros(), len, "zeros leading_zeros len={len}");
        assert_eq!(z.leading_ones(), 0, "zeros leading_ones len={len}");
        assert_eq!(z.trailing_zeros(), len, "zeros trailing_zeros len={len}");
        assert_eq!(z.trailing_ones(), 0, "zeros trailing_ones len={len}");

        let o = BitString::ones(len);
        assert_eq!(o.leading_zeros(), 0, "ones leading_zeros len={len}");
        assert_eq!(o.leading_ones(), len, "ones leading_ones len={len}");
        assert_eq!(o.trailing_zeros(), 0, "ones trailing_zeros len={len}");
        assert_eq!(o.trailing_ones(), len, "ones trailing_ones len={len}");

        // Alternating starting with 0: "010101..."
        let alt0 = {
            let s = "01".repeat((len + 1) / 2);
            bs(&s[..len])
        };
        assert_eq!(
            alt0.leading_zeros(),
            if len > 0 { 1 } else { 0 },
            "0101... leading_zeros len={len}"
        );
        assert_eq!(alt0.leading_ones(), 0, "0101... leading_ones len={len}");

        // Alternating starting with 1: "101010..."
        let alt1 = {
            let s = "10".repeat((len + 1) / 2);
            bs(&s[..len])
        };
        assert_eq!(alt1.leading_zeros(), 0, "1010... leading_zeros len={len}");
        assert_eq!(
            alt1.leading_ones(),
            if len > 0 { 1 } else { 0 },
            "1010... leading_ones len={len}"
        );
    }
}

// ===========================================================================
// B. Single-bit and edge-position attacks
// ===========================================================================

#[test]
fn attack_bitstring_leading_trailing_single_bit_positions() {
    // Place a single 1-bit at every position from 0 to 255 and verify
    // all four counts from both ends.
    for pos in 0..256 {
        let len = pos + 1;
        let mut bits = BitString::zeros(len);
        // Set the bit at `pos` (0 = MSB / leftmost in the string).
        // BitString index 0 is the leftmost bit.
        bits.set(pos, true);

        // Leading zeros: must be `pos` (all bits before position pos are 0).
        assert_eq!(
            bits.leading_zeros(),
            pos,
            "single 1 at pos={pos} leading_zeros"
        );
        // Leading ones: always 0 (bit 0 is never 1 unless pos==0).
        if pos == 0 {
            assert_eq!(bits.leading_ones(), 1);
        } else {
            assert_eq!(bits.leading_ones(), 0);
        }

        // Trailing zeros: bits after pos are all 0.
        assert_eq!(
            bits.trailing_zeros(),
            len - pos - 1,
            "single 1 at pos={pos} trailing_zeros len={len}"
        );
        // Trailing ones: always 0 unless pos is the last bit.
        if pos == len - 1 {
            assert_eq!(bits.trailing_ones(), 1);
        } else {
            assert_eq!(bits.trailing_ones(), 0);
        }
    }
}

// ===========================================================================
// C. Cross-word boundary patterns
// ===========================================================================

#[test]
fn attack_bitstring_leading_crosses_word_boundary() {
    // Leading FILL spans exactly k words, followed by a non-FILL bit.
    // Test the SIMD scan and the scalar tail around word boundaries.
    for prepend_words in [0, 1, 2, 3, 4, 5, 8, 9] {
        for extra_bits in [0, 1, 63] {
            let total_zeros = prepend_words * 64 + extra_bits;
            if total_zeros == 0 {
                continue;
            }
            let total_len = total_zeros + 1;
            let mut bits = BitString::zeros(total_len);
            // Flip the first non-zero bit to 1 — this is where leading_zeros stops.
            bits.set(total_zeros, true);
            assert_eq!(
                bits.leading_zeros(),
                total_zeros,
                "leading_zeros: {} words + {} bits of zeros then 1",
                prepend_words,
                extra_bits
            );
            assert_eq!(bits.leading_ones(), 0);
        }
    }
}

#[test]
fn attack_bitstring_trailing_crosses_word_boundary() {
    // Trailing FILL spans exactly k words counting backwards.
    for append_words in [0, 1, 2, 3, 4, 5, 8, 9] {
        for extra_bits in [0, 1, 63] {
            let total_zeros = append_words * 64 + extra_bits;
            if total_zeros == 0 {
                continue;
            }
            let total_len = total_zeros + 1;
            let mut bits = BitString::zeros(total_len);
            // Set the leftmost bit to 1 — trailing_zeros counts from the right.
            bits.set(0, true);
            assert_eq!(
                bits.trailing_zeros(),
                total_zeros,
                "trailing_zeros: 1 then {} words + {} bits of zeros",
                append_words,
                extra_bits
            );
            assert_eq!(bits.trailing_ones(), 0);
        }
    }
}

#[test]
fn attack_bitstring_leading_trailing_boundary_shifts() {
    // Test patterns where the FILL/non-FILL transition lands exactly
    // at ±1 from every word boundary up to 4 words.
    for word_boundary in [63, 64, 65, 127, 128, 129, 191, 192, 193] {
        let len = word_boundary + 10;

        // Leading: zeros up to `word_boundary`, then a 1.
        let mut bits = BitString::zeros(len);
        bits.set(word_boundary, true);
        assert_eq!(
            bits.leading_zeros(),
            word_boundary,
            "leading_zeros boundary={word_boundary}"
        );

        // Leading with ones.
        let mut bits = BitString::ones(len);
        bits.set(word_boundary, false);
        assert_eq!(
            bits.leading_ones(),
            word_boundary,
            "leading_ones boundary={word_boundary}"
        );
    }
}

// ===========================================================================
// D. Last-word masking — unused bits must not affect results
// ===========================================================================

#[test]
fn attack_last_word_mask_does_not_affect_counts() {
    // Create bitstrings of every length 1..256, fill them entirely with
    // either zeros or ones, then verify counts.  The last-word mask must
    // zero out bits beyond `bit_len`, otherwise leading_zeros /
    // trailing_zeros on a genuinely all-zero string would return the
    // wrong answer.
    for len in 1..=256 {
        // All zeros — mask must prevent trailing junk from adding spurious
        // leading/trailing zeros.
        let z = BitString::zeros(len);
        assert_eq!(z.leading_zeros(), len, "zeros leading_zeros len={len}");
        assert_eq!(z.trailing_zeros(), len, "zeros trailing_zeros len={len}");

        // All ones — mask must prevent trailing junk from adding spurious
        // leading/trailing ones.
        let o = BitString::ones(len);
        assert_eq!(o.leading_ones(), len, "ones leading_ones len={len}");
        assert_eq!(o.trailing_ones(), len, "ones trailing_ones len={len}");

        // Verify that the invariant holds: unused bits in the last word
        // are actually zeroed.
        assert!(view_has_same_invariants(&z));
        assert!(view_has_same_invariants(&o));
    }
}

// ===========================================================================
// E. BitStr oracle — systematic misaligned view comparison
// ===========================================================================

#[test]
fn attack_bitstr_leading_trailing_oracle_misaligned() {
    // For patterns long enough to span multiple words, test every
    // possible start offset (0..64) combined with every bit length
    // (0..remaining).  Compare the BitStr result against the
    // round-tripped BitString result.
    let patterns: Vec<(&str, BitString)> = vec![
        ("zeros", BitString::zeros(300)),
        ("ones", BitString::ones(300)),
        ("0101", bs(&"01".repeat(150))),
        ("1010", bs(&"10".repeat(150))),
        (
            "0x20_1_zeros",
            bs(&cat(&[
                "0".repeat(20).as_str(),
                "1",
                "0".repeat(279).as_str(),
            ])),
        ),
        (
            "0x20_1_ones",
            bs(&cat(&[
                "1".repeat(20).as_str(),
                "0",
                "1".repeat(279).as_str(),
            ])),
        ),
    ];

    for (name, bits) in &patterns {
        let view = bits.as_bit_str();
        for start in 0..64.min(bits.bit_len()) {
            let max_len = bits.bit_len() - start;
            for len in 1..=max_len.min(130) {
                let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
                let sub = view.slice(interval);
                let owned = sub.to_bit_string();

                assert_eq!(
                    sub.leading_zeros(),
                    owned.leading_zeros(),
                    "BitStr leading_zeros mismatch pattern={name} start={start} len={len}"
                );
                assert_eq!(
                    sub.leading_ones(),
                    owned.leading_ones(),
                    "BitStr leading_ones mismatch pattern={name} start={start} len={len}"
                );
                assert_eq!(
                    sub.trailing_zeros(),
                    owned.trailing_zeros(),
                    "BitStr trailing_zeros mismatch pattern={name} start={start} len={len}"
                );
                assert_eq!(
                    sub.trailing_ones(),
                    owned.trailing_ones(),
                    "BitStr trailing_ones mismatch pattern={name} start={start} len={len}"
                );
            }
        }
    }
}

// ===========================================================================
// F. BitStr misaligned — both start and end unaligned
// ===========================================================================

#[test]
fn attack_bitstr_misaligned_both_ends() {
    // Construct a long pattern, then take views where both `start` and
    // `start + len` are unaligned (not mod 64).  This stresses the
    // first-word and last-word partial handling simultaneously.
    let body = "01".repeat(200); // 400 bits
    let bits = bs(&body);
    let view = bits.as_bit_str();

    // Test every misaligned start offset (1..63) with varied lengths.
    for start in 1..64 {
        for len_parts in [0, 1, 2, 3, 5, 8] {
            let len = len_parts * 64 + 37; // deliberately unaligned length
            if start + len > 400 {
                continue;
            }
            let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
            let sub = view.slice(interval);
            let owned = sub.to_bit_string();

            assert_eq!(
                sub.leading_zeros(),
                owned.leading_zeros(),
                "misaligned-both leading_zeros start={start} len={len}"
            );
            assert_eq!(
                sub.leading_ones(),
                owned.leading_ones(),
                "misaligned-both leading_ones start={start} len={len}"
            );
            assert_eq!(
                sub.trailing_zeros(),
                owned.trailing_zeros(),
                "misaligned-both trailing_zeros start={start} len={len}"
            );
            assert_eq!(
                sub.trailing_ones(),
                owned.trailing_ones(),
                "misaligned-both trailing_ones start={start} len={len}"
            );
        }
    }
}

// ===========================================================================
// G. Single-word views — short bit lengths with various offsets
// ===========================================================================

#[test]
fn attack_bitstr_single_word_views() {
    // Breach a view entirely inside one word of the source, with every
    // possible misalignment within that word.
    let bits = bs(&cat(&[
        "0".repeat(100).as_str(),
        "1".repeat(100).as_str(),
        "0".repeat(100).as_str(),
    ]));
    let view = bits.as_bit_str();

    // Test every offset within a single word, for lengths that stay
    // within one word.
    for start in 0..100 {
        for len in 1..=(100 - start).min(64) {
            let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
            let sub = view.slice(interval);
            let owned = sub.to_bit_string();

            assert_eq!(
                sub.leading_zeros(),
                owned.leading_zeros(),
                "single-word leading_zeros start={start} len={len}"
            );
            assert_eq!(
                sub.trailing_zeros(),
                owned.trailing_zeros(),
                "single-word trailing_zeros start={start} len={len}"
            );
            assert_eq!(
                sub.leading_ones(),
                owned.leading_ones(),
                "single-word leading_ones start={start} len={len}"
            );
            assert_eq!(
                sub.trailing_ones(),
                owned.trailing_ones(),
                "single-word trailing_ones start={start} len={len}"
            );
        }
    }
}

// ===========================================================================
// H. BitString leading_ones / trailing_ones on non-trivial patterns
// ===========================================================================

// ===========================================================================
// I. Zero-length edge cases
// ===========================================================================

#[test]
fn attack_leading_trailing_zero_length() {
    // BitString
    let z = BitString::zeros(0);
    assert_eq!(z.leading_zeros(), 0);
    assert_eq!(z.leading_ones(), 0);
    assert_eq!(z.trailing_zeros(), 0);
    assert_eq!(z.trailing_ones(), 0);

    // BitStr view of zero length — slice_from at the very end creates
    // an empty view.
    let bits = bs("01010101");
    let view = bits.as_bit_str();
    let empty = view.slice_from(8);
    assert_eq!(empty.leading_zeros(), 0);
    assert_eq!(empty.leading_ones(), 0);
    assert_eq!(empty.trailing_zeros(), 0);
    assert_eq!(empty.trailing_ones(), 0);
    // Empty view at a misaligned position within the backing word.
    let long = bs(&"01".repeat(100)); // 200 bits
    let lv = long.as_bit_str();
    let empty_misaligned = lv.slice_from(200);
    assert_eq!(empty_misaligned.bit_len(), 0);
    assert_eq!(empty_misaligned.leading_zeros(), 0);
    assert_eq!(empty_misaligned.leading_ones(), 0);
    assert_eq!(empty_misaligned.trailing_zeros(), 0);
    assert_eq!(empty_misaligned.trailing_ones(), 0);
}

#[test]
fn attack_bitstring_leading_trailing_ones_complex() {
    // Leading ones: k ones followed by a zero.
    for k in [1, 2, 63, 64, 65, 127, 128, 129] {
        let mut bits = BitString::ones(k + 1);
        bits.set(k, false);
        assert_eq!(
            bits.leading_ones(),
            k,
            "leading_ones len={} with zero at pos={k}",
            k + 1
        );
    }

    // Trailing ones: a zero followed by k ones.
    for k in [1, 2, 63, 64, 65, 127, 128, 129] {
        let len = k + 1;
        let mut bits = BitString::ones(len);
        bits.set(0, false);
        assert_eq!(
            bits.trailing_ones(),
            k,
            "trailing_ones len={len} with zero at pos=0"
        );
    }
}
