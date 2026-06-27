use super::*;

#[test]
fn attack_matches_at_oob() {
    let bits = bs("10101");
    let pattern = bits.as_bit_str();

    // index beyond len
    assert!(!bits.matches_at(5, pattern));
    assert!(!bits.matches_at(usize::MAX, pattern));

    // pattern longer than remaining
    let binding = bs("101010");
    let pattern = binding.as_bit_str(); // len 6
    assert!(!bits.matches_at(0, pattern));

    // Empty pattern always matches
    let empty_binding = BitString::new();
    let empty = empty_binding.as_bit_str();
    assert!(bits.matches_at(0, empty));
    assert!(bits.matches_at(5, empty));
}

#[test]
fn attack_starts_with_ends_with_edge() {
    let bits = bs("10101");

    // Empty always matches
    assert!(bits.starts_with(BitString::new().as_bit_str()));
    assert!(bits.ends_with(BitString::new().as_bit_str()));

    // Longer than self
    assert!(!bits.starts_with(bs("101010").as_bit_str()));
    assert!(!bits.ends_with(bs("101010").as_bit_str()));

    // Exact match
    assert!(bits.starts_with(bits.as_bit_str()));
    assert!(bits.ends_with(bits.as_bit_str()));
}

#[test]
fn attack_find_empty_needle() {
    let bits = bs("10101");
    let empty_binding2 = BitString::new();
    let needle = empty_binding2.as_bit_str();

    assert_eq!(bits.find(needle), Some(0));
    assert_eq!(bits.rfind(needle), Some(5)); // rfind with empty returns len
}

#[test]
fn attack_find_needle_longer_than_haystack() {
    let bits = bs("10");
    let binding3 = bs("101");
    let needle = binding3.as_bit_str();
    assert_eq!(bits.find(needle), None);
    assert_eq!(bits.rfind(needle), None);
    assert!(!bits.contains(needle));
}

#[test]
fn attack_find_at_word_boundaries() {
    // "0" * 128 + "101" + "0" * 128
    let mut bits = BitString::zeros(128);
    bits.push_bit_string(&bs("101"));
    bits.push_bit_string(&BitString::zeros(128));
    let binding4 = bs("101");
    let needle = binding4.as_bit_str();

    assert_eq!(bits.find(needle), Some(128));
    assert_eq!(bits.rfind(needle), Some(128));
    assert!(bits.contains(needle));
}

#[test]
fn attack_find_pattern_at_word_edge() {
    // Pattern that straddles word boundary
    for offset in [32, 60, 61, 62, 63, 64, 65, 66, 67, 95, 96, 100] {
        let mut bits = BitString::zeros(256);
        let pattern = bs("110011");
        bits.replace_assign(offset, &pattern);

        let found = bits.find(pattern.as_bit_str());
        assert_eq!(found, Some(offset), "find failed at offset {offset}");

        let rfound = bits.rfind(pattern.as_bit_str());
        assert_eq!(rfound, Some(offset), "rfind failed at offset {offset}");
    }
}

#[test]
fn attack_find_multiple_occurrences() {
    let bits = bs(&("101".to_owned() + &"0".repeat(60) + "101" + &"0".repeat(60) + "101"));
    let binding5 = bs("101");
    let needle = binding5.as_bit_str();

    assert_eq!(bits.find(needle), Some(0));
    assert_eq!(bits.rfind(needle), Some(126)); // 0 + 3 + 60 + 3 + 60 = 126

    // Find all occurrences
    let mut pos = 0;
    let mut count = 0;
    while let Some(p) = bits.as_bit_str().slice_from(pos).find(needle) {
        count += 1;
        pos += p + needle.bit_len();
    }
    assert_eq!(count, 3);
}

#[test]
fn attack_strip_prefix_suffix() {
    let bits = bs("10101");

    // Strip empty
    assert_eq!(
        bits.strip_prefix(BitString::new().as_bit_str()).unwrap(),
        bits
    );
    assert_eq!(
        bits.strip_suffix(BitString::new().as_bit_str()).unwrap(),
        bits
    );

    // Mismatch
    assert!(bits.strip_prefix(bs("0").as_bit_str()).is_none());
    assert!(bits.strip_suffix(bs("0").as_bit_str()).is_none());

    // Valid strip
    let stripped = bits.strip_prefix(bs("10").as_bit_str()).unwrap();
    assert_eq!(stripped.to_string(), "101");

    let stripped = bits.strip_suffix(bs("01").as_bit_str()).unwrap();
    assert_eq!(stripped.to_string(), "101");
}
