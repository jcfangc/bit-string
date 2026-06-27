use crate::BitString;

/// Reversing the empty string produces the empty string.
#[test]
fn reverse_empty() {
    let bits = BitString::new();
    assert_eq!(bits.reverse_bits(), bits);
}

/// Reversing a single bit is a no-op.
#[test]
fn reverse_single_bit() {
    for val in [false, true] {
        let bits = BitString::repeat(val, 1);
        let rev = bits.reverse_bits();
        assert_eq!(rev, bits, "val={val}");
    }
}

/// Reversing a known pattern.
#[test]
fn reverse_known_pattern() {
    let bits = BitString::try_from("10011").unwrap();
    let rev = bits.reverse_bits();
    let expected = BitString::try_from("11001").unwrap();
    assert_eq!(rev, expected);

    // Reverse twice should produce original.
    assert_eq!(rev.reverse_bits(), bits);
}

/// Reversing "1010" → "0101"
#[test]
fn reverse_alternating() {
    let bits = BitString::try_from("1010").unwrap();
    let rev = bits.reverse_bits();
    assert_eq!(rev, BitString::try_from("0101").unwrap());
    assert_eq!(rev.reverse_bits(), bits);
}

/// Reverse a palindrome → same.
#[test]
fn reverse_palindrome() {
    let bits = BitString::try_from("1001").unwrap();
    assert_eq!(bits.reverse_bits(), bits);
}

/// All zeros.
#[test]
fn reverse_all_zeros() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::zeros(len);
        assert_eq!(bits.reverse_bits(), bits, "len={len}");
    }
}

/// All ones.
#[test]
fn reverse_all_ones() {
    for len in [1, 63, 64, 65, 127, 128, 129, 130] {
        let bits = BitString::ones(len);
        assert_eq!(bits.reverse_bits(), bits, "len={len}");
    }
}

/// Reverse a string with a single 1 at various positions.
#[test]
fn reverse_single_one_at_various_positions() {
    for pos in [0, 1, 31, 63, 64, 65, 100, 127, 128, 129] {
        let len = 130;
        let mut bits = BitString::zeros(len);
        bits.set(pos, true);
        let rev = bits.reverse_bits();
        // The reversed bit should be at position len - 1 - pos.
        assert!(rev.get(len - 1 - pos).unwrap(), "pos={pos}");
        // All other positions should be 0.
        assert_eq!(rev.count_ones(), 1, "pos={pos}");
    }
}

/// Reverse twice → identity (invariant).
#[test]
fn reverse_twice_is_identity() {
    for len in [1, 2, 5, 63, 64, 65, 127, 128, 129, 130] {
        let mut bits = BitString::zeros(len);
        for i in (0..len).step_by(3) {
            bits.set(i, true);
        }
        assert_eq!(bits.reverse_bits().reverse_bits(), bits, "len={len}");
    }
}

/// reverse_bits_assign modifies in place.
#[test]
fn reverse_bits_assign_modifies_in_place() {
    let mut bits = BitString::try_from("10011").unwrap();
    let expected = bits.reverse_bits();
    bits.reverse_bits_assign();
    assert_eq!(bits, expected);
}

/// Reverse preserves bit_len.
#[test]
fn reverse_preserves_bit_len() {
    for len in [0, 1, 2, 63, 64, 65, 128, 129] {
        let mut bits = BitString::zeros(len);
        for i in (0..len).step_by(5) {
            bits.set(i, true);
        }
        assert_eq!(bits.reverse_bits().bit_len(), len, "len={len}");
    }
}
