use super::*;

#[test]
fn attack_push_bit_string_self() {
    // Self-append should work (we clone the source first)
    let mut bits = bs("101");
    let clone = bits.clone();
    bits.push_bit_string(&clone);
    assert_eq!(bits.to_string(), "101101");
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_push_bit_string_empty() {
    let mut bits = bs("101");
    let empty = BitString::new();
    bits.push_bit_string(&empty);
    assert_eq!(bits.to_string(), "101");
    assert!(view_has_same_invariants(&bits));

    // Empty + non-empty
    let mut empty = BitString::new();
    let other = bs("11");
    empty.push_bit_string(&other);
    assert_eq!(empty.to_string(), "11");
    assert!(view_has_same_invariants(&empty));
}

#[test]
fn attack_push_bit_string_word_boundaries() {
    // Every combination of lengths around word boundaries
    for a_len in [0, 63, 64, 65] {
        for b_len in [0, 1, 63, 64, 65, 127] {
            let a = BitString::ones(a_len);
            let b = BitString::zeros(b_len);
            let mut c = a.clone();
            c.push_bit_string(&b);

            let expected_len = a_len + b_len;
            assert_eq!(c.bit_len(), expected_len, "len: {a_len}+{b_len}");

            // First a_len bits should be ones
            for i in 0..a_len {
                assert_eq!(c.get(i), Some(true), "pos {i}/{a_len}+{b_len}");
            }
            // Remaining should be zeros
            for i in a_len..expected_len {
                assert_eq!(c.get(i), Some(false), "pos {i}/{a_len}+{b_len}");
            }
            assert!(view_has_same_invariants(&c));
        }
    }
}

#[test]
fn attack_insert_bit_string_self() {
    let bits = bs("111");
    let clone = bits.clone();
    let mut result = bits.clone();
    result.insert_bit_string(1, &clone); // insert "111" at index 1
    assert_eq!(result.to_string(), "111111"); // "1" + "111" + "11" = 6 bits
    assert!(view_has_same_invariants(&result));
}

#[test]
fn attack_insert_bit_string_boundaries() {
    let mut bits = BitString::new();

    // Insert into empty at index 0
    bits.insert_bit_string(0, &bs("101"));
    assert_eq!(bits.to_string(), "101");

    // Insert at beginning
    bits.insert_bit_string(0, &bs("01"));
    assert_eq!(bits.to_string(), "01101");

    // Insert at end
    bits.insert_bit_string(usize::MAX, &bs("10"));
    assert_eq!(bits.to_string(), "0110110");

    // Insert empty string
    let len_before = bits.bit_len();
    bits.insert_bit_string(2, &BitString::new());
    assert_eq!(bits.bit_len(), len_before);

    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_insert_bit_string_word_boundaries() {
    let filler = BitString::ones(40);
    for host_len in [63, 64, 65] {
        let host = BitString::zeros(host_len);
        for at in [0, host_len / 2, host_len] {
            let mut bits = host.clone();
            bits.insert_bit_string(at, &filler);
            assert_eq!(bits.bit_len(), host_len + 40);
            assert!(view_has_same_invariants(&bits));

            // 40 ones should be at `at`
            for i in at..at + 40 {
                assert_eq!(
                    bits.get(i),
                    Some(true),
                    "bit {i} should be 1, host_len={host_len}, at={at}"
                );
            }
        }
    }
}

#[test]
fn attack_split_off_boundaries() {
    let mut bits = bs("1010101"); // len 7

    // Split at len
    let tail = bits.split_off(7);
    assert!(tail.is_empty());
    assert_eq!(bits.to_string(), "1010101");

    // Split at usize::MAX (clamped to 7)
    let tail = bits.split_off(usize::MAX);
    assert!(tail.is_empty());
    assert_eq!(bits.to_string(), "1010101");

    // Split at 0
    let tail = bits.split_off(0);
    assert_eq!(tail.to_string(), "1010101");
    assert!(bits.is_empty());
    assert!(view_has_same_invariants(&tail));

    bits = bs("1010101");
    // Split in middle
    let tail = bits.split_off(3);
    assert_eq!(bits.to_string(), "101");
    assert_eq!(tail.to_string(), "0101");
    assert!(view_has_same_invariants(&bits));
    assert!(view_has_same_invariants(&tail));
}

#[test]
fn attack_split_off_word_boundaries() {
    for len in [63, 64, 65, 127, 128, 129] {
        let mut bits = BitString::ones(len);
        for at in [0, 1, 32, 63, 64, 65, len] {
            if at > len {
                continue;
            }
            let tail = bits.split_off(at);
            assert_eq!(bits.bit_len(), at, "len={len}, at={at}");
            assert_eq!(tail.bit_len(), len - at, "len={len}, at={at}");
            assert!(view_has_same_invariants(&bits));
            assert!(view_has_same_invariants(&tail));

            // Reassemble and verify
            bits.push_bit_string(&tail);
            assert_eq!(bits.bit_len(), len);
            assert_eq!(bits.count_ones(), len);

            // Reset for next iteration
            bits = BitString::ones(len);
        }
    }
}
