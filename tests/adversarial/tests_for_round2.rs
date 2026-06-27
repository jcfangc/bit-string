use super::*;
use core::cmp::Ordering;
use int_interval::UsizeCO;

// Helper: concatenate string slices into a String
fn cat(parts: &[&str]) -> String {
    parts.iter().copied().collect()
}

// ===========================================================================
// A. Both-sides-unaligned Eq / Ord / Hash
// ===========================================================================

#[test]
fn attack_eq_both_unaligned_same_bits() {
    // "0"*10 + "1111111111" + "0"*10 = 30 bits.  The middle ones block is at [10,20).
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
    ]));
    let b = a.clone();

    // Both sub-views at different unaligned offsets, both within the ones block.
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 8).unwrap()); // "11111111"
    let vb = b
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(12, 8).unwrap()); // "11111111"
    assert_eq!(va, vb);
    assert_eq!(va.to_bit_string(), vb.to_bit_string());
}

#[test]
fn attack_eq_both_unaligned_different_bits() {
    // "1"*20 + "0"*20.  Offset 1 → "1111", offset 19 → "1000".
    let a = bs(&cat(&["1".repeat(20).as_str(), "0".repeat(20).as_str()]));
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(1, 4).unwrap()); // "1111"
    let vb = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(19, 4).unwrap()); // "1000"
    assert_ne!(va, vb);
}

#[test]
fn attack_ord_both_unaligned() {
    // "1"*30 + "0"*30.   "11" vs "10" from different unaligned offsets.
    let a = bs(&cat(&["1".repeat(30).as_str(), "0".repeat(30).as_str()]));
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(3, 2).unwrap()); // "11"
    let vb = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(29, 2).unwrap()); // "10"
    assert!(va > vb);
    assert_eq!(va.partial_cmp(&vb), Some(Ordering::Greater));
}

#[test]
fn attack_ord_both_unaligned_equal() {
    // "0"*5 + "111111" + "0"*5 = 16 bits.  "1111" at offsets 5 and 7.
    let a = bs(&cat(&[
        "0".repeat(5).as_str(),
        "111111",
        "0".repeat(5).as_str(),
    ]));
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(5, 4).unwrap()); // "1111"
    let vb = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(7, 4).unwrap()); // "1111"
    assert_eq!(va.cmp(&vb), Ordering::Equal);
}

#[test]
fn attack_hash_both_unaligned_same_bits() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
    ]));
    let b = a.clone();
    let va = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 8).unwrap());
    let vb = b
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(12, 8).unwrap());
    assert_eq!(va, vb);
    assert_eq!(hash(&va), hash(&vb));
}

#[test]
fn attack_hash_unaligned_vs_aligned() {
    let a = bs(&cat(&[
        "0".repeat(20).as_str(),
        "11001100",
        "0".repeat(20).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(20, 8).unwrap());
    let owned = bs("11001100");
    assert_eq!(view, owned.as_bit_str());
    assert_eq!(hash(&view), hash(&owned.as_bit_str()));
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
    assert_eq!(view.find(needle.as_bit_str()), Some(7));
    assert_eq!(view.rfind(needle.as_bit_str()), Some(7));
    assert!(view.contains(needle.as_bit_str()));
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
    assert_eq!(hay_view.find(needle_view), Some(4));
    assert!(hay_view.contains(needle_view));
}

#[test]
#[ignore = "BUG B4: rfind misses last occurrence when aligned portion < SMALL_WORDS words"]
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
    assert_eq!(view.find(needle.as_bit_str()), Some(3));
    // BUG: rfind only scans first partial word when aligned portion < SMALL_WORDS
    assert_eq!(view.rfind(needle.as_bit_str()), Some(129));
    let mut pos = 0;
    let mut count = 0;
    while let Some(p) = view.slice_from(pos).find(needle.as_bit_str()) {
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
    assert!(view.contains(bs("101").as_bit_str()));
    assert!(!view.contains(bs("111").as_bit_str()));
}

// ===========================================================================
// C. Structural vs semantic Eq
// ===========================================================================

#[test]
fn attack_structural_eq_agrees_with_semantic_eq() {
    for len in [0, 1, 63, 64, 65, 128] {
        let a = BitString::ones(len);
        let b = BitString::ones(len);
        assert_eq!(a, b);
        assert_eq!(a.as_bit_str(), b.as_bit_str());

        let mut c = a.clone();
        if len > 2 {
            c.set(1, false);
        }
        let d = c.clone();
        assert_eq!(c, d);
        assert_eq!(c.as_bit_str(), d.as_bit_str());
    }
}

#[test]
fn attack_structural_eq_with_extra_capacity() {
    let mut a = BitString::zeros(10);
    a.push_bit_string(&BitString::zeros(100));
    a.truncate(10);
    let b = BitString::zeros(10);
    assert_eq!(a, b);
    assert_eq!(a.as_bit_str(), b.as_bit_str());
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

// ===========================================================================
// E. get_chunk on unaligned BitStr views
// ===========================================================================

#[test]
fn attack_get_chunk_unaligned_view() {
    let a = bs(&cat(&[
        "0".repeat(20).as_str(),
        "1".repeat(20).as_str(),
        "0".repeat(20).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(17, 40).unwrap());
    assert_eq!(view.get_chunk(0) & 0b111, 0b000);
    assert_eq!(view.get_chunk(3) & ((1u64 << 17) - 1), (1u64 << 17) - 1);
    assert_eq!(view.get_chunk(40), 0);
}

#[test]
fn attack_get_chunk_partial_last_word() {
    let a = bs(&cat(&["0".repeat(30).as_str(), "10101010"]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(31, 4).unwrap());
    assert_eq!(view.get_chunk(0) & 0xF, 0b1010);
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
    assert!(hay_view.matches_at(3, nd_view));
    assert!(hay_view.matches_at(7, nd_view));
    // Wrong position: "1100" at offset 5
    assert!(!hay_view.matches_at(5, nd_view));
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
    assert!(view.starts_with(nd_view));
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
    assert!(view.ends_with(nd_view));
}

// ===========================================================================
// H. retain at word boundaries
// ===========================================================================

#[test]
fn attack_retain_crossing_word_boundary() {
    let mut bits = BitString::zeros(200);
    for i in (63..200).step_by(3) {
        bits.set(i, true);
    }
    let ones_count_before = bits.count_ones();
    bits.retain(|b| b);
    assert_eq!(bits.count_ones(), ones_count_before);
    assert!(view_has_same_invariants(&bits));
    assert!(bits.is_all_ones());
}

#[test]
fn attack_retain_all_false_clears_long() {
    let mut bits = BitString::ones(130);
    bits.retain(|_| false);
    assert!(bits.is_empty());
    assert!(view_has_same_invariants(&bits));
}

// ===========================================================================
// I. set_chunk edge cases
// ===========================================================================

#[test]
fn attack_set_chunk_len_zero() {
    let mut bits = bs("10101");
    let before = bits.to_string();
    bits.set_chunk(0, u64::MAX, 0);
    assert_eq!(bits.to_string(), before);
    assert!(view_has_same_invariants(&bits));
}

#[test]
fn attack_set_chunk_at_word_boundary() {
    let mut bits = BitString::zeros(128);
    bits.set_chunk(64, 0xAAAA, 16);
    assert!(view_has_same_invariants(&bits));
    let chunk = bits.get_chunk(64);
    assert_eq!(chunk & 0xFFFF, 0xAAAA);
    for i in 0..64 {
        assert_eq!(bits.get(i), Some(false));
    }
    for i in 80..128 {
        assert_eq!(bits.get(i), Some(false));
    }
}

// ===========================================================================
// J. BitStr::iter on unaligned views
// ===========================================================================

#[test]
fn attack_bitstr_iter_unaligned() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "10101",
        "0".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(11, 5).unwrap());
    let collected: Vec<bool> = view.iter().collect();
    assert_eq!(collected, vec![false, true, false, true, false]);
}

#[test]
fn attack_bitstr_iter_double_ended_unaligned() {
    let a = bs(&cat(&[
        "1".repeat(5).as_str(),
        "00110011",
        "1".repeat(5).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(6, 6).unwrap());
    let mut iter = view.iter();
    assert_eq!(iter.next(), Some(false));
    assert_eq!(iter.next_back(), Some(true));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), Some(true));
    assert_eq!(iter.next_back(), Some(false));
    assert_eq!(iter.next(), None);
}

// ===========================================================================
// K. BitStr predicates on unaligned views
// ===========================================================================

#[test]
fn attack_predicates_unaligned_all_ones() {
    let a = bs(&cat(&[
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 10).unwrap());
    assert!(view.all());
    assert!(view.is_all_ones());
    assert!(!view.is_all_zeros());
    assert!(view.any());
}

#[test]
fn attack_predicates_unaligned_all_zeros() {
    let a = bs(&cat(&[
        "1".repeat(10).as_str(),
        "0".repeat(10).as_str(),
        "1".repeat(10).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(10, 10).unwrap());
    assert!(!view.any());
    assert!(!view.all());
    assert!(view.is_all_zeros());
}

#[test]
fn attack_predicates_unaligned_empty_view() {
    let bits = bs("10101");
    // empty view via sliced at boundary
    let v1 = bits.as_bit_str().slice_from(5);
    assert!(v1.is_empty());
    assert!(v1.all());
    assert!(v1.is_all_zeros());
    assert!(v1.is_all_ones());
    assert!(!v1.any());
}

// ===========================================================================
// L. BitStr Display/Debug on unaligned views
// ===========================================================================

#[test]
fn attack_bitstr_display_unaligned() {
    let a = bs(&cat(&[
        "0".repeat(5).as_str(),
        "1100",
        "1".repeat(5).as_str(),
    ]));
    let view = a
        .as_bit_str()
        .slice(UsizeCO::checked_from_start_len(5, 4).unwrap());
    assert_eq!(format!("{}", view), "1100");
    assert!(format!("{:?}", view).contains("1100"));
}

// ===========================================================================
// M. push/pop/set_chunk at word boundaries
// ===========================================================================

#[test]
fn attack_push_pop_at_all_word_boundaries() {
    for target_len in [63, 64, 65, 127, 128, 129] {
        let mut bits = BitString::zeros(target_len);
        bits.push(true);
        assert_eq!(bits.bit_len(), target_len + 1);
        assert!(view_has_same_invariants(&bits));
        assert_eq!(bits.get(target_len), Some(true));
        assert_eq!(bits.pop(), Some(true));
        assert_eq!(bits.bit_len(), target_len);
        assert!(view_has_same_invariants(&bits));
    }
}

#[test]
fn attack_set_chunk_exactly_filling_word() {
    let mut bits = BitString::zeros(64);
    bits.set_chunk(0, u64::MAX, 64);
    assert_eq!(bits, BitString::ones(64));
    assert!(view_has_same_invariants(&bits));
    bits.set_chunk(64, u64::MAX, 64);
    assert_eq!(bits, BitString::ones(64));
    assert!(view_has_same_invariants(&bits));
}
