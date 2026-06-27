use super::*;
use int_interval::UsizeCO;

#[test]
fn attack_replace_equal_length() {
    // Replace with same-length — should be in-place
    let mut bits = bs("11110000");
    let repl = bs("0011");
    bits.replace_assign(2, &repl);
    assert_eq!(bits.to_string(), "11001100");
    assert!(view_has_same_invariants(&bits));

    // Replace via interval
    let bits = bs("11110000");
    let result = bits.replace_interval(UsizeCO::checked_from_start_len(2, 4).unwrap(), &repl);
    assert_eq!(result.to_string(), "11001100");
    assert!(view_has_same_invariants(&result));
}

#[test]
fn attack_replace_different_lengths() {
    // Replace with shorter — replace 1 bit at index 2 with "0"
    // Original: "11110000" => replace bit 2 (which is '1') with '0'
    // Result: "11010000"
    let mut bits = bs("11110000");
    let repl = bs("0");
    bits.replace_assign(2, &repl);
    assert_eq!(bits.to_string(), "11010000");
    assert!(view_has_same_invariants(&bits));

    // Replace with longer (clamped range < replacement length)
    let mut bits = bs("11000");
    let repl = bs("111");
    bits.replace_assign(4, &repl); // clamped range [4,5) = 1 bit, repl = 3 bits => grow
    assert_eq!(bits.to_string(), "1100111");
    assert!(view_has_same_invariants(&bits));

    // Replace with empty replacement at edge => effectively insert
    let mut bits = bs("10101");
    bits.replace_assign(0, &BitString::new());
    assert_eq!(bits.to_string(), "10101"); // empty replacement, no-op
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_replace_at_word_boundaries() {
    for len in [63, 64, 65] {
        let host = BitString::ones(len);
        let repl = BitString::zeros(5);
        for at in [0, len / 2, len.saturating_sub(5)] {
            let result = host.replace(at, &repl);
            assert_eq!(result.bit_len(), len); // same length
            assert!(view_has_same_invariants(&result));

            // Check the zeros are at the right place
            for i in at..at + 5 {
                assert_eq!(
                    result.get(i),
                    Some(false),
                    "should be 0 at {i}, len={len}, at={at}"
                );
            }
        }
    }
}

#[test]
fn attack_replace_self_as_replacement() {
    // Replace using self as replacement (should clone internally)
    let bits = bs("110011");
    let result = bits.replace_interval(UsizeCO::checked_from_start_len(2, 2).unwrap(), &bits);
    // Removed "00" (positions 2-3), inserted "110011"
    assert_eq!(result.to_string(), "1111001111");
    assert!(view_has_same_invariants(&result));
}
