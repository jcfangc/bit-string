use super::*;
use int_interval::UsizeCO;

#[test]
fn attack_matches_at_oob() {
    let bits = bs("10101");
    let pattern = bits.as_bit_str();

    // index beyond len
    assert!(!bits.matches_at_str(5, pattern));
    assert!(!bits.matches_at_str(usize::MAX, pattern));

    // pattern longer than remaining
    let binding = bs("101010");
    let pattern = binding.as_bit_str(); // len 6
    assert!(!bits.matches_at_str(0, pattern));

    // Empty pattern always matches
    let empty_binding = BitString::new();
    let empty = empty_binding.as_bit_str();
    assert!(bits.matches_at_str(0, empty));
    assert!(bits.matches_at_str(5, empty));
}

#[test]
fn attack_starts_with_ends_with_edge() {
    let bits = bs("10101");

    // Empty always matches
    assert!(bits.starts_with_str(BitString::new().as_bit_str()));
    assert!(bits.ends_with_str(BitString::new().as_bit_str()));

    // Longer than self
    assert!(!bits.starts_with_str(bs("101010").as_bit_str()));
    assert!(!bits.ends_with_str(bs("101010").as_bit_str()));

    // Exact match
    assert!(bits.starts_with_str(bits.as_bit_str()));
    assert!(bits.ends_with_str(bits.as_bit_str()));
}

#[test]
fn attack_find_empty_needle() {
    let bits = bs("10101");
    let empty_binding2 = BitString::new();
    let needle = empty_binding2.as_bit_str();

    assert_eq!(bits.find_str(needle), Some(0));
    assert_eq!(bits.rfind_str(needle), Some(5)); // rfind with empty returns len
}

#[test]
fn attack_find_needle_longer_than_haystack() {
    let bits = bs("10");
    let binding3 = bs("101");
    let needle = binding3.as_bit_str();
    assert_eq!(bits.find_str(needle), None);
    assert_eq!(bits.rfind_str(needle), None);
    assert!(!bits.contains_str(needle));
}

#[test]
fn attack_find_at_word_boundaries() {
    // "0" * 128 + "101" + "0" * 128
    let mut bits = BitString::zeros(128);
    bits.push_bit_string(&bs("101"));
    bits.push_bit_string(&BitString::zeros(128));
    let binding4 = bs("101");
    let needle = binding4.as_bit_str();

    assert_eq!(bits.find_str(needle), Some(128));
    assert_eq!(bits.rfind_str(needle), Some(128));
    assert!(bits.contains_str(needle));
}

#[test]
fn attack_find_pattern_at_word_edge() {
    // Pattern that straddles word boundary
    for offset in [32, 60, 61, 62, 63, 64, 65, 66, 67, 95, 96, 100] {
        let mut bits = BitString::zeros(256);
        let pattern = bs("110011");
        bits.replace_assign(offset, &pattern);

        let found = bits.find_str(pattern.as_bit_str());
        assert_eq!(found, Some(offset), "find failed at offset {offset}");

        let rfound = bits.rfind_str(pattern.as_bit_str());
        assert_eq!(rfound, Some(offset), "rfind failed at offset {offset}");
    }
}

#[test]
fn attack_find_multiple_occurrences() {
    let bits = bs(&("101".to_owned() + &"0".repeat(60) + "101" + &"0".repeat(60) + "101"));
    let binding5 = bs("101");
    let needle = binding5.as_bit_str();

    assert_eq!(bits.find_str(needle), Some(0));
    assert_eq!(bits.rfind_str(needle), Some(126)); // 0 + 3 + 60 + 3 + 60 = 126

    // Find all occurrences
    let mut pos = 0;
    let mut count = 0;
    while let Some(p) = bits.as_bit_str().slice_from(pos).find_str(needle) {
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

// ===========================================================================
// B. Unaligned find / contains / rfind
// ===========================================================================

#[test]
fn attack_find_on_unaligned_haystack_aligned_needle() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "101",
        "0".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(3, 20).unwrap());
    let needle = bs("101");
    assert_eq!(view.find_str(needle.as_bit_str()), Some(7));
    assert_eq!(view.rfind_str(needle.as_bit_str()), Some(7));
    assert!(view.contains_str(needle.as_bit_str()));
}

#[test]
fn attack_find_on_unaligned_haystack_unaligned_needle() {
    let src = bs(&cat(&[
        "0".repeat(9).as_str(),
        "110011",
        "0".repeat(30).as_str(),
    ]));
    let needle_view = src
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 4).unwrap());
    let hay = bs(&cat(&[
        "1".repeat(5).as_str(),
        "1001",
        "0".repeat(10).as_str(),
    ]));
    let hay_view = hay
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(1, 18).unwrap());
    assert_eq!(hay_view.find_str(needle_view), Some(4));
    assert!(hay_view.contains_str(needle_view));
}

#[test]
fn attack_find_unaligned_multiple_occurrences() {
    let s = cat(&[
        "0".repeat(5).as_str(),
        "101",
        "0".repeat(60).as_str(),
        "101",
        "0".repeat(60).as_str(),
        "101",
    ]);
    let bits = bs(&s);
    let view = bits
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(2, 132).unwrap());
    let needle = bs("101");
    assert_eq!(view.find_str(needle.as_bit_str()), Some(3));
    assert_eq!(view.rfind_str(needle.as_bit_str()), Some(129));
    let mut pos = 0;
    let mut count = 0;
    while let Some(p) = view.slice_from(pos).find_str(needle.as_bit_str()) {
        count += 1;
        pos += p + 3;
    }
    assert_eq!(count, 3);
}

#[test]
fn attack_contains_on_unaligned_short_haystack() {
    let a = bs(&cat(&[
        "0".repeat(40).as_str(),
        "101",
        "0".repeat(40).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(37, 20).unwrap());
    assert!(view.contains_str(bs("101").as_bit_str()));
    assert!(!view.contains_str(bs("111").as_bit_str()));
}

// ===========================================================================
// F. strip_prefix / strip_suffix on unaligned views
// ===========================================================================

#[test]
fn attack_strip_prefix_unaligned() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "11001100",
        "0".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(3, 20).unwrap());
    let result = view.strip_prefix(bs("0000000").as_bit_str());
    assert!(result.is_some());
    assert_eq!(result.unwrap().to_bit_string().to_string(), "1100110000000");
}

#[test]
fn attack_strip_suffix_unaligned() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "101101",
        "0".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(7, 15).unwrap());
    let result = view.strip_suffix(bs("000000").as_bit_str());
    assert!(result.is_some());
    assert_eq!(result.unwrap().to_bit_string().to_string(), "000101101");
}

// ===========================================================================
// G. matches_at / starts_with / ends_with both-unaligned
// ===========================================================================

#[test]
fn attack_matches_at_both_unaligned() {
    // Source: "0"*10 + "11001100" + "0"*30.  "11001100" at [10,18).
    // Haystack view at offset 5, len 20: positions 5-24 = "00000" + "11001100" + "0000000"
    //   "0011" is at relative offset 7 within this view (positions 12-15 of source).
    let src = bs(&cat(&[
        "0".repeat(10).as_str(),
        "11001100",
        "0".repeat(30).as_str(),
    ]));
    let hay_view = src
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(5, 20).unwrap());

    // Needle from a different unaligned view: "1001" at offset 6 of "11111" + "1001" + "11111"
    // Source: "1"*5 + "1001" + "1"*5.  Offset 6, len 4 = "0011".
    let needle_src = bs(&cat(&[
        "1".repeat(5).as_str(),
        "1001",
        "1".repeat(5).as_str(),
    ]));
    let nd_view = needle_src
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(6, 4).unwrap()); // "0011"

    // "0011" at relative offset 3 (source 8-11 = "0011") and offset 7 (source 12-15)
    assert!(hay_view.matches_at_str(3, nd_view));
    assert!(hay_view.matches_at_str(7, nd_view));
    // Wrong position: "1100" at offset 5
    assert!(!hay_view.matches_at_str(5, nd_view));
}

#[test]
fn attack_starts_with_both_unaligned() {
    let src = bs("110011001100");
    let view = src
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(3, 6).unwrap());
    let nd_src = bs(&cat(&[
        "0".repeat(2).as_str(),
        "0110",
        "0".repeat(2).as_str(),
    ]));
    let nd_view = nd_src
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(2, 4).unwrap());
    assert!(view.starts_with_str(nd_view));
}

#[test]
fn attack_ends_with_both_unaligned() {
    let src = bs("110011001100");
    let view = src
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(2, 8).unwrap());
    let nd_src = bs(&("1".repeat(3) + "0011"));
    let nd_view = nd_src
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(3, 4).unwrap());
    assert!(view.ends_with_str(nd_view));
}
