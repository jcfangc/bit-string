use super::*;

#[test]
fn attack_empty_construction() {
    let a = BitString::new();
    assert!(a.is_empty());
    assert_eq!(a.bit_len(), 0);
    assert!(a.words().is_empty());
    assert!(view_has_same_invariants(&a));

    let b = BitString::default();
    assert_eq!(a, b);

    let c = BitString::zeros(0);
    assert_eq!(a, c);

    let d = BitString::ones(0);
    assert_eq!(a, d);

    let e = BitString::repeat(true, 0);
    let f = BitString::repeat(false, 0);
    assert_eq!(e, f);
}

#[test]
fn attack_from_words_dirty_high_bits() {
    // from_words should mask dirty high bits in the last word
    let words = &[u64::MAX, u64::MAX];
    let bits = BitString::from_words(words, 65).unwrap();
    assert!(view_has_same_invariants(&bits));

    // The 65th bit (bit 1 of second word) should be present, bits 66+ zeroed
    assert_eq!(bits.get(64), Some(true));
    assert_eq!(bits.get(65), None);

    // Verify all bits beyond len are reported as None
    for i in 65..128 {
        assert!(bits.get(i).is_none(), "bit {i} should be out of range");
    }
}

#[test]
fn attack_from_words_wrong_count() {
    // Too few words
    assert!(BitString::from_words(&[0u64], 65).is_none());
    // Too many words
    assert!(BitString::from_words(&[0u64, 0], 63).is_none());
    // Zero length with non-empty words
    assert!(BitString::from_words(&[1u64], 0).is_none());
    // Empty words with non-zero length
    assert!(BitString::from_words(&[], 1).is_none());
}

#[test]
fn attack_from_str_invalid() {
    assert!("".parse::<BitString>().is_ok()); // empty is valid
    assert!("0".parse::<BitString>().is_ok());
    assert!("1".parse::<BitString>().is_ok());
    assert!("2".parse::<BitString>().is_err());
    assert!("xyz".parse::<BitString>().is_err());
    assert!("0101x010".parse::<BitString>().is_err());
    assert!(" ".parse::<BitString>().is_err());
    assert!("\0".parse::<BitString>().is_err());
    assert!("\n".parse::<BitString>().is_err());
    // Unicode
    assert!("🔥".parse::<BitString>().is_err());
    // Very long valid string
    let s = "01".repeat(10_000);
    assert!(s.parse::<BitString>().is_ok());
}

#[test]
fn attack_from_bool_slice_empty() {
    let a = BitString::from(&[] as &[bool]);
    assert!(a.is_empty());

    let b = BitString::from([false; 0]);
    assert!(b.is_empty());

    let c: BitString = [].into_iter().collect();
    assert!(c.is_empty());
}

#[test]
fn attack_from_bool_iter_large() {
    let many: Vec<bool> = (0..100_000).map(|i| i % 3 == 0).collect();
    let bits = BitString::from_iter(many.iter().copied());
    assert_eq!(bits.bit_len(), 100_000);
    assert!(view_has_same_invariants(&bits));

    // Spot-check
    for i in [0, 3, 6, 99999] {
        let expected = i % 3 == 0;
        assert_eq!(bits.get(i), Some(expected), "mismatch at index {i}");
    }
}

#[test]
fn attack_zeros_ones_word_boundaries() {
    for len in [0, 1, 63, 64, 65, 127, 128, 129, 1023, 1024, 1025] {
        let zeros = BitString::zeros(len);
        assert!(view_has_same_invariants(&zeros));
        assert_eq!(zeros.count_ones(), 0);
        assert_eq!(zeros.bit_len(), len);

        let ones = BitString::ones(len);
        assert!(view_has_same_invariants(&ones));
        assert_eq!(ones.count_ones(), len);
        assert_eq!(ones.bit_len(), len);

        let rep = BitString::repeat(true, len);
        assert_eq!(rep, ones);
    }
}
