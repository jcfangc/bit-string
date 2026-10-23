use super::PackedString;
use bit_string::{AnchorHint, ContainsOptions, FindOptions, LocationHint, traits::PackedChar};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Code<const BITS: u8>(u8);

impl<const BITS: u8> PackedChar<BITS> for Code<BITS> {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        let mask = ((1u16 << BITS) - 1) as u8;
        (code & !mask == 0).then_some(Self(code))
    }
}

fn check_options_for_width<const BITS: u8>() {
    let mask = ((1u16 << BITS) - 1) as u8;
    let haystack_codes: Vec<u8> = (0..160)
        .map(|index| ((index * 17 + index / 3 + 5) as u8) & mask)
        .collect();
    let needle_codes = haystack_codes[71..84].to_vec();
    let haystack =
        PackedString::<Code<BITS>, BITS>::from_chars(haystack_codes.iter().copied().map(Code));
    let needle =
        PackedString::<Code<BITS>, BITS>::from_chars(needle_codes.iter().copied().map(Code));
    let absent = PackedString::<Code<BITS>, BITS>::from_chars(core::iter::repeat_n(Code(mask), 4));
    let oversized = PackedString::<Code<BITS>, BITS>::from_chars(core::iter::repeat_n(
        Code(mask),
        haystack_codes.len() + 1,
    ));

    // A nonzero character offset makes the borrowed view word-unaligned for
    // every supported width while preserving character alignment.
    let view_start = 1;
    let view_len = haystack_codes.len() - 2;
    let haystack_view = haystack
        .as_packed_str()
        .slice_from(view_start)
        .slice_until(view_start + view_len);
    let needle_view = needle.as_packed_str();
    let expected_find = haystack_codes[view_start..view_start + view_len]
        .windows(needle_codes.len())
        .position(|window| window == needle_codes);
    let expected_rfind = haystack_codes[view_start..view_start + view_len]
        .windows(needle_codes.len())
        .rposition(|window| window == needle_codes);
    let expected_owned_find = haystack_codes
        .windows(needle_codes.len())
        .position(|window| window == needle_codes);
    let expected_owned_rfind = haystack_codes
        .windows(needle_codes.len())
        .rposition(|window| window == needle_codes);

    for anchor in [AnchorHint::Unknown, AnchorHint::Prefix, AnchorHint::Suffix] {
        let options = FindOptions::default().with_anchor(anchor);
        assert_eq!(haystack_view.find_with(needle_view, options), expected_find);
        assert_eq!(
            haystack_view.rfind_with(needle_view, options),
            expected_rfind
        );
        assert_eq!(haystack.find_with(&needle, options), expected_owned_find);
        assert_eq!(haystack.rfind_with(&needle, options), expected_owned_rfind);
        assert_eq!(
            haystack_view.find_with(absent.as_packed_str(), options),
            None
        );
        assert_eq!(
            haystack_view.rfind_with(absent.as_packed_str(), options),
            None
        );
        assert_eq!(haystack.find_with(&oversized, options), None);
        assert_eq!(haystack.rfind_with(&oversized, options), None);
    }

    for anchor in [AnchorHint::Unknown, AnchorHint::Prefix, AnchorHint::Suffix] {
        for location in [
            LocationHint::Unknown,
            LocationHint::NearStart,
            LocationHint::NearEnd,
        ] {
            let options = ContainsOptions::default()
                .with_anchor(anchor)
                .with_location(location);
            assert_eq!(
                haystack_view.contains_with(needle_view, options),
                expected_find.is_some()
            );
            assert_eq!(
                haystack.contains_with(&needle, options),
                expected_find.is_some()
            );
            assert!(!haystack_view.contains_with(absent.as_packed_str(), options));
            assert!(!haystack.contains_with(&oversized, options));
        }
    }

    let empty = PackedString::<Code<BITS>, BITS>::new();
    for anchor in [AnchorHint::Unknown, AnchorHint::Prefix, AnchorHint::Suffix] {
        let options = FindOptions::default().with_anchor(anchor);
        assert_eq!(empty.find_with(&empty, options), Some(0));
        assert_eq!(empty.rfind_with(&empty, options), Some(0));
    }
    for anchor in [AnchorHint::Unknown, AnchorHint::Prefix, AnchorHint::Suffix] {
        for location in [
            LocationHint::Unknown,
            LocationHint::NearStart,
            LocationHint::NearEnd,
        ] {
            assert!(
                empty.contains_with(
                    &empty,
                    ContainsOptions::default()
                        .with_anchor(anchor)
                        .with_location(location)
                )
            );
        }
    }
}

#[test]
fn eight_bit_dual_anchor_threshold_matches_the_character_model() {
    const HAYSTACK_LEN: usize = 128;
    let mask = u8::MAX;

    for needle_len in [23, 24, 25] {
        let mut haystack_codes: Vec<u8> = (0..HAYSTACK_LEN)
            .map(|index| (index * 73 + index / 2 + 17) as u8)
            .collect();
        let needle_codes = haystack_codes[40..40 + needle_len].to_vec();
        let last_start = HAYSTACK_LEN - needle_len;
        haystack_codes[2..2 + needle_len].copy_from_slice(&needle_codes);
        haystack_codes[last_start..].copy_from_slice(&needle_codes);

        let haystack =
            PackedString::<Code<8>, 8>::from_chars(haystack_codes.iter().copied().map(Code::<8>));
        let needle =
            PackedString::<Code<8>, 8>::from_chars(needle_codes.iter().copied().map(Code::<8>));
        let absent_codes = vec![mask; needle_len];
        let absent =
            PackedString::<Code<8>, 8>::from_chars(absent_codes.iter().copied().map(Code::<8>));

        let mut backing_codes = Vec::with_capacity(HAYSTACK_LEN + 2);
        backing_codes.push(0xa5);
        backing_codes.extend_from_slice(&haystack_codes);
        backing_codes.push(0x5a);
        let backing =
            PackedString::<Code<8>, 8>::from_chars(backing_codes.iter().copied().map(Code::<8>));
        let view = backing
            .as_packed_str()
            .slice_from(1)
            .slice_until(HAYSTACK_LEN + 1);

        let expected_find = haystack_codes
            .windows(needle_len)
            .position(|window| window == needle_codes);
        let expected_rfind = haystack_codes
            .windows(needle_len)
            .rposition(|window| window == needle_codes);
        assert_eq!(expected_find, Some(2));
        assert_eq!(expected_rfind, Some(last_start));
        assert_eq!(
            haystack_codes
                .windows(needle_len)
                .position(|window| window == absent_codes),
            None
        );

        for anchor in [AnchorHint::Unknown, AnchorHint::Prefix, AnchorHint::Suffix] {
            let options = FindOptions::default().with_anchor(anchor);
            assert_eq!(haystack.find_with(&needle, options), expected_find);
            assert_eq!(haystack.rfind_with(&needle, options), expected_rfind);
            assert_eq!(
                view.find_with(needle.as_packed_str(), options),
                expected_find
            );
            assert_eq!(
                view.rfind_with(needle.as_packed_str(), options),
                expected_rfind
            );
            assert_eq!(haystack.find_with(&absent, options), None);
            assert_eq!(haystack.rfind_with(&absent, options), None);
        }

        for anchor in [AnchorHint::Unknown, AnchorHint::Prefix, AnchorHint::Suffix] {
            for location in [
                LocationHint::Unknown,
                LocationHint::NearStart,
                LocationHint::NearEnd,
            ] {
                let options = ContainsOptions::default()
                    .with_anchor(anchor)
                    .with_location(location);
                assert!(haystack.contains_with(&needle, options));
                assert!(view.contains_with(needle.as_packed_str(), options));
                assert!(!haystack.contains_with(&absent, options));
            }
        }
    }
}

#[test]
fn packed_search_hints_never_change_results() {
    check_options_for_width::<1>();
    check_options_for_width::<2>();
    check_options_for_width::<3>();
    check_options_for_width::<4>();
    check_options_for_width::<5>();
    check_options_for_width::<6>();
    check_options_for_width::<7>();
    check_options_for_width::<8>();
}
