use super::*;
use int_interval::UsizeCO;

#[test]
fn attack_clone_invariants() {
    for len in [0, 1, 63, 64, 65, 127, 128, 129] {
        let bits = BitString::ones(len);
        let clone = bits.clone();
        assert_eq!(bits, clone);
        assert!(view_has_same_invariants(&clone));

        // Clone should be independent
        if len > 0 {
            let mut clone2 = bits.clone();
            clone2.set(0, false).unwrap();
            assert_ne!(bits, clone2);
        }
    }
}

#[test]
fn attack_bitstr_to_bit_string_word_boundaries() {
    // to_bit_string on sub-views at various offsets
    let bits = BitString::ones(200);
    let view = bits.as_bit_str();

    for start in [0, 1, 32, 63, 64, 65, 100, 127, 128, 129, 200] {
        for len in [1, 63, 64, 65] {
            if start + len > 200 {
                continue;
            }
            let sub = view.slice(UsizeCO::checked_from_start_len(start, len).unwrap());
            let owned = sub.to_bit_string();
            assert_eq!(owned.bit_len(), len);
            assert!(view_has_same_invariants(&owned));
            assert_eq!(owned.count_ones(), len);
        }
    }
}

#[test]
fn attack_bitstr_source_after_mutation() {
    // BitStr borrows source; verify it's correct even after the source is
    // consumed (but not mutated, since BitStr borrows it).
    let bits = bs("11001100");
    let view = bits.as_bit_str();

    // Create sub-views
    let sub = view.slice(UsizeCO::checked_from_start_len(2, 4).unwrap());
    assert_eq!(sub.to_bit_string().to_string(), "0011");

    // All views should be consistent
    assert_eq!(view.bit_len(), 8);
}

#[test]
fn attack_bitstr_slice_chain() {
    let bits = bs("1100110011");
    let v = bits.as_bit_str();
    let v1 = v.slice_from(2); // "00110011"
    let v2 = v1.slice_until(6); // "001100"
    let v3 = v2.slice(UsizeCO::checked_from_start_len(2, 2).unwrap()); // "11"

    assert_eq!(v3.to_bit_string().to_string(), "11");
    assert_eq!(v3.start(), 4); // 0 + 2 + 2 = 4
    assert_eq!(v3.bit_len(), 2);
}
