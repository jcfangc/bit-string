use super::*;
use int_interval::UsizeCO;

#[test]
fn attack_count_ones_zeros() {
    for len in [0, 1, 63, 64, 65, 127, 128, 129] {
        let zeros = BitString::zeros(len);
        assert_eq!(zeros.count_ones(), 0);
        assert_eq!(zeros.count_zeros(), len);

        let ones = BitString::ones(len);
        assert_eq!(ones.count_ones(), len);
        assert_eq!(ones.count_zeros(), 0);

        let _mixed = BitString::repeat(true, len);
        let _ = _mixed; // placeholder
    }
}

#[test]
fn attack_leading_zeros_word_boundaries() {
    // Leading zeros on word-aligned views
    for len in [0, 1, 64, 65, 128, 129] {
        let bits = BitString::zeros(len);
        assert_eq!(
            bits.leading_zeros(),
            len,
            "leading_zeros on zeros len={len}"
        );
        assert_eq!(bits.leading_ones(), 0, "leading_ones on zeros len={len}");
        assert_eq!(
            bits.trailing_zeros(),
            len,
            "trailing_zeros on zeros len={len}"
        );
        assert_eq!(bits.trailing_ones(), 0, "trailing_ones on zeros len={len}");

        let bits = BitString::ones(len);
        assert_eq!(bits.leading_zeros(), 0);
        assert_eq!(bits.leading_ones(), len);
        assert_eq!(bits.trailing_zeros(), 0);
        assert_eq!(bits.trailing_ones(), len);
    }
}

#[test]
fn attack_leading_zeros_mixed() {
    // "0011100" — 2 leading zeros, 1 trailing? No, 2 trailing zeros
    let bits = bs("0011100");
    assert_eq!(bits.leading_zeros(), 2);
    assert_eq!(bits.leading_ones(), 0);
    assert_eq!(bits.trailing_zeros(), 2);
    assert_eq!(bits.trailing_ones(), 0);

    // "1110001" — 3 leading ones, 0 trailing zeros? No, 1 trailing one (since end is "1")
    // Wait: "1110001" = positions: 0=1, 1=1, 2=1, 3=0, 4=0, 5=0, 6=1
    let bits = bs("1110001");
    assert_eq!(bits.leading_zeros(), 0);
    assert_eq!(bits.leading_ones(), 3);
    assert_eq!(bits.trailing_zeros(), 0);
    assert_eq!(bits.trailing_ones(), 1);

    // Single bit
    let bits = bs("0");
    assert_eq!(bits.leading_zeros(), 1);
    assert_eq!(bits.trailing_zeros(), 1);

    let bits = bs("1");
    assert_eq!(bits.leading_ones(), 1);
    assert_eq!(bits.trailing_ones(), 1);
}

#[test]
fn attack_bitstr_view_leading_trailing() {
    // Create a bitstring, take a sub-view, check leading/trailing on the view
    let bits = bs("000111000");
    let view = bits.as_bit_str();

    // view of "00111" at offset 1, len 5
    let sub = view.slice(UsizeCO::checked_from_start_len(1, 5).unwrap());
    assert_eq!(sub.to_bit_string().to_string(), "00111");
    assert_eq!(sub.leading_zeros(), 2);
    assert_eq!(sub.leading_ones(), 0);
    assert_eq!(sub.trailing_zeros(), 0);
    assert_eq!(sub.trailing_ones(), 3);

    // Misaligned start within a word: offset 3, len 5 from the original
    // "11100" from bits "000111000" at offset 3
    let sub2 = view.slice(UsizeCO::checked_from_start_len(3, 5).unwrap());
    assert_eq!(sub2.to_bit_string().to_string(), "11100");
    assert_eq!(sub2.leading_zeros(), 0);
    assert_eq!(sub2.leading_ones(), 3);
    assert_eq!(sub2.trailing_zeros(), 2);
    assert_eq!(sub2.trailing_ones(), 0);
}

#[test]
fn attack_bitstr_count_ones_misaligned() {
    // Test count_ones on BitStr views with unaligned starts
    let bits = bs(&("0".repeat(200) + "1" + &"0".repeat(63)));
    let view = bits.as_bit_str();

    // Slice that starts and ends at various offsets
    for start in 0..200 {
        for len in 1..64.min(264 - start) {
            let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
            let sub = view.slice(interval);
            let owned = sub.to_bit_string();
            assert_eq!(
                sub.count_ones(),
                owned.count_ones(),
                "count_ones mismatch at start={start}, len={len}"
            );
            assert_eq!(
                sub.count_zeros(),
                owned.count_zeros(),
                "count_zeros mismatch at start={start}, len={len}"
            );
        }
    }
}

#[test]
fn attack_count_ones_at_all_offsets() {
    // Systematic test: pattern "010101..." at every offset for every length
    let pattern = "01".repeat(10); // 20 bits
    let bits = bs(&pattern);
    let view = bits.as_bit_str();

    for start in 0..20 {
        for len in 1..=20 - start {
            let interval = UsizeCO::checked_from_start_len(start, len).unwrap();
            let sub = view.slice(interval);
            let owned = sub.to_bit_string();
            assert_eq!(
                sub.count_ones(),
                owned.count_ones(),
                "count_ones mismatch at start={start}, len={len}, pattern={pattern}"
            );
        }
    }
}

// ===========================================================================
// D. trailing_zeros / trailing_ones on unaligned BitStr views
// ===========================================================================

#[test]
fn attack_trailing_zeros_unaligned() {
    let a = bs(&cat(&[
        "1".repeat(30).as_str(),
        "0".repeat(5).as_str(),
        "1".repeat(30).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(17, 30).unwrap());
    assert_eq!(view.trailing_zeros(), 0);
    assert_eq!(view.trailing_ones(), 12);
}

#[test]
fn attack_trailing_zeros_unaligned_single_word() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "1".repeat(5).as_str(),
        "0".repeat(50).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(13, 3).unwrap());
    assert_eq!(view.trailing_zeros(), 1);
    assert_eq!(view.trailing_ones(), 0);
}

#[test]
fn attack_leading_zeros_unaligned_single_word() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "001",
        "1".repeat(50).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 3).unwrap());
    assert_eq!(view.leading_zeros(), 2);
    assert_eq!(view.leading_ones(), 0);
}
