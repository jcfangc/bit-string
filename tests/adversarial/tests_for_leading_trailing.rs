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

// ===========================================================================
// J. Exhaustive leading_ones / trailing_ones — all lengths 0..256
// ===========================================================================

#[test]
fn attack_bitstring_leading_trailing_ones_exhaustive() {
    // Mirror of Section A: all-ones background with a single zero at each
    // position, verifying both leading_ones and trailing_ones.
    for len in 0..=256 {
        // Pure ones
        let all = BitString::ones(len);
        assert_eq!(all.leading_ones(), len, "all-ones leading_ones len={len}");
        assert_eq!(all.trailing_ones(), len, "all-ones trailing_ones len={len}");

        // Single zero at each position
        for pos in 0..len {
            let mut bits = BitString::ones(len);
            bits.set(pos, false);

            let expected_leading = if pos == 0 { 0 } else { pos };
            let expected_trailing = len - pos - 1;
            assert_eq!(
                bits.leading_ones(),
                expected_leading,
                "leading_ones len={len} zero@pos={pos}"
            );
            assert_eq!(
                bits.trailing_ones(),
                expected_trailing,
                "trailing_ones len={len} zero@pos={pos}"
            );
        }
    }
}

// ===========================================================================
// K. Very long strings — exercise SIMD + ALIGN_THRESHOLD boundary
// ===========================================================================

#[test]
fn attack_bitstring_leading_long_simd() {
    // Exercise the AVX2 aligned-load path (total >= 128 words = 8192 bits)
    // and the SSE2 unaligned path.  Vary lengths above and below
    // alignment-sensitive boundaries.
    let lengths: &[usize] = &[
        256,   // well below ALIGN_THRESHOLD
        1023,  // just under 1k bits
        4096,  // 64 words — SSE2 territory, small AVX2 path
        8192,  // 128 words — ALIGN_THRESHOLD boundary
        8193,  // just over boundary
        10000, // odd size
        16384, // 256 words
        32768, // 512 words
        65535, // odd large
        65536, // 1024 words — well above threshold
    ];
    for &len in lengths {
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

        // Single break near start / middle / end
        for pos in [0, 1, 63, 64, 65, len / 2, len - 2, len - 1] {
            if pos >= len {
                continue;
            }
            let mut bits = BitString::ones(len);
            bits.set(pos, false);
            assert_eq!(
                bits.leading_ones(),
                pos,
                "len={len} zero@pos={pos} leading_ones"
            );
            assert_eq!(
                bits.trailing_ones(),
                len - pos - 1,
                "len={len} zero@pos={pos} trailing_ones"
            );
        }
    }
}

// ===========================================================================
// L. BitStr deep unaligned — start beyond first word, unaligned offset
// ===========================================================================

#[test]
fn attack_bitstr_deep_unaligned_views() {
    // Create a long backing string, then take sub-views with various
    // start offsets (including deep into the backing) and lengths,
    // verifying against a round-tripped BitString.
    let backing_len = 2048;
    let backing = {
        let mut bs = BitString::zeros(backing_len);
        // Put a pattern: 128 zeros, then alternating groups of 64 ones / 64 zeros
        for i in 128..backing_len {
            bs.set(i, (i / 64) % 2 == 1);
        }
        bs
    };
    let full = backing.as_bit_str();

    // Non-word-aligned start offsets deep in the backing
    let start_offsets: &[usize] = &[0, 1, 3, 31, 63, 64, 65, 100, 127, 128, 129, 500, 1023];
    for &start in start_offsets {
        for &len in &[1, 2, 31, 63, 64, 65, 127, 128, 129, 256] {
            if start + len > backing_len {
                continue;
            }
            let view = full.slice(UsizeCO::try_new(start, start + len).unwrap());
            // Round-trip oracle
            let rt = bs(&view.to_string());
            assert_eq!(
                view.leading_zeros(),
                rt.leading_zeros(),
                "leading_zeros start={start} len={len}"
            );
            assert_eq!(
                view.leading_ones(),
                rt.leading_ones(),
                "leading_ones start={start} len={len}"
            );
            assert_eq!(
                view.trailing_zeros(),
                rt.trailing_zeros(),
                "trailing_zeros start={start} len={len}"
            );
            assert_eq!(
                view.trailing_ones(),
                rt.trailing_ones(),
                "trailing_ones start={start} len={len}"
            );
        }
    }

    // All-zeros and all-ones sub-views at deep unaligned starts
    for &start in &[0, 7, 63, 64, 65, 128, 129, 500] {
        let zs = BitString::zeros(backing_len);
        let zf = zs.as_bit_str();
        for &len in &[1, 63, 64, 65, 128, 256] {
            if start + len > backing_len {
                continue;
            }
            let zv = zf.slice(UsizeCO::try_new(start, start + len).unwrap());
            assert_eq!(
                zv.leading_zeros(),
                len,
                "all-zeros leading_zeros start={start} len={len}"
            );
            assert_eq!(
                zv.trailing_zeros(),
                len,
                "all-zeros trailing_zeros start={start} len={len}"
            );
            assert_eq!(
                zv.leading_ones(),
                0,
                "all-zeros leading_ones start={start} len={len}"
            );
        }
    }
}

// ===========================================================================
// M. Leading/trailing cross-word boundary — ones variant
// ===========================================================================

#[test]
fn attack_bitstring_leading_trailing_ones_cross_word() {
    // Mirror Section C for ones: all-ones background with a zero that
    // crosses word boundaries.
    for n_words in 0..10 {
        for extra_bits in [0, 1, 63] {
            let k = n_words * 64 + extra_bits;
            if k == 0 {
                continue;
            }

            // leading_ones: k ones, then a zero
            let mut bits = BitString::ones(k + 1);
            bits.set(k, false);
            assert_eq!(
                bits.leading_ones(),
                k,
                "leading_ones k={k} ({} words + {} bits)",
                n_words,
                extra_bits
            );

            // trailing_ones: a zero, then k ones
            let mut bits = BitString::ones(k + 1);
            bits.set(0, false);
            assert_eq!(
                bits.trailing_ones(),
                k,
                "trailing_ones k={k} ({} words + {} bits)",
                n_words,
                extra_bits
            );
        }
    }
}

// ===========================================================================
// N. Randomized cross-operation invariants
// ===========================================================================

/// Simple LCG for deterministic pseudo-random bits.
fn lcg_step(state: &mut u64) {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
}

#[test]
fn attack_leading_trailing_random_invariants() {
    // Generate 200 pseudo-random BitStrings and verify cross-operation
    // invariants on both BitString and BitStr views.
    let mut rng: u64 = 0xdead_beef_cafe_babe;
    for _ in 0..200 {
        lcg_step(&mut rng);
        let seed = rng;
        lcg_step(&mut rng);
        let len = (rng as usize % 1024) + 1;
        let mut bits = BitString::zeros(len);
        for i in 0..len {
            lcg_step(&mut rng);
            bits.set(i, rng & 1 != 0);
        }

        // BitString invariants
        let lz = bits.leading_zeros();
        let lo = bits.leading_ones();
        let tz = bits.trailing_zeros();
        let to = bits.trailing_ones();
        assert!(lz <= len, "lz={lz} > len={len} seed={seed}");
        assert!(lo <= len, "lo={lo} > len={len} seed={seed}");
        assert!(tz <= len, "tz={tz} > len={len} seed={seed}");
        assert!(to <= len, "to={to} > len={len} seed={seed}");
        // At least one of lz, lo is 0 (first bit is either 0 or 1)
        assert!(
            lz == 0 || lo == 0,
            "lz={lz} lo={lo} seed={seed} — first bit cannot be both 0 and 1"
        );
        // At least one of tz, to is 0 (last bit is either 0 or 1)
        assert!(
            tz == 0 || to == 0,
            "tz={tz} to={to} seed={seed} — last bit cannot be both 0 and 1"
        );

        // BitStr oracle comparison for random sub-views
        let full = bits.as_bit_str();
        lcg_step(&mut rng);
        let start = (rng as usize) % len;
        lcg_step(&mut rng);
        let sub_len = if start < len {
            ((rng as usize) % (len - start)).max(1)
        } else {
            1
        };
        if start + sub_len <= len {
            let view = full.slice(UsizeCO::try_new(start, start + sub_len).unwrap());
            let rt = bs(&view.to_string());
            assert_eq!(
                view.leading_zeros(),
                rt.leading_zeros(),
                "random view lz seed={seed} start={start} len={sub_len}"
            );
            assert_eq!(
                view.trailing_zeros(),
                rt.trailing_zeros(),
                "random view tz seed={seed} start={start} len={sub_len}"
            );
            assert_eq!(
                view.leading_ones(),
                rt.leading_ones(),
                "random view lo seed={seed} start={start} len={sub_len}"
            );
            assert_eq!(
                view.trailing_ones(),
                rt.trailing_ones(),
                "random view to seed={seed} start={start} len={sub_len}"
            );
        }
    }
}

// ===========================================================================
// O. Stress interleaving mutation with leading/trailing counts
// ===========================================================================

#[test]
fn attack_leading_trailing_after_mutation() {
    // Verify that mutating a BitString (set, push, pop, slice, truncate)
    // produces correct counts afterward.  This stresses the last-word
    // mask invariant and boundary recalculations.
    for word_len in [1, 2, 63, 64, 65, 128, 129] {
        // Build: word_len zeros, then a single 1
        let mut bits = BitString::zeros(word_len + 1);
        bits.set(word_len, true);

        // push more zeros after the 1
        bits.push(false);
        bits.push(false);
        assert_eq!(
            bits.leading_zeros(),
            word_len,
            "after push zeros, word_len={word_len}"
        );
        assert_eq!(
            bits.trailing_zeros(),
            2,
            "after push zeros trailing, word_len={word_len}"
        );

        // pop them back
        bits.pop();
        bits.pop();
        assert_eq!(
            bits.leading_zeros(),
            word_len,
            "after pop, word_len={word_len}"
        );
        assert_eq!(
            bits.trailing_zeros(),
            0,
            "after pop trailing, word_len={word_len}"
        );

        // truncate to the 1-bit
        bits.truncate(word_len);
        assert_eq!(
            bits.leading_zeros(),
            word_len,
            "after truncate, word_len={word_len}"
        );
        assert_eq!(
            bits.trailing_zeros(),
            word_len,
            "after truncate trailing, word_len={word_len}"
        );

        // set a bit near a word boundary
        bits.set(0, true);
        assert_eq!(
            bits.leading_zeros(),
            0,
            "after set pos=0, word_len={word_len}"
        );
        assert_eq!(
            bits.leading_ones(),
            1,
            "after set pos=0 ones, word_len={word_len}"
        );
    }
}
