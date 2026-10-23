//! Paired measurements for packed `contains_with` hints.
//!
//! Run with `cargo bench --bench search_options_hints`. Each benchmark compares
//! one deterministic haystack/needle pair per seed. Fixture construction and
//! oracle checks happen outside the timed closure. Case IDs are head hit,
//! tail hit, absent with a selective prefix, and absent with a selective
//! suffix. For hit cases, the correct and incorrect profiles differ only in
//! expected location; for absent cases they differ only in expected anchor.

use bit_string::{AnchorHint, ContainsOptions, LocationHint, PackedString};
use divan::{Bencher, black_box};

#[path = "packed_support/mod.rs"]
mod support;

const HAYSTACK_LEN: usize = 65_536;
const NEEDLE_LEN: usize = 48;
const SEEDS: [u64; 4] = [
    0x243f_6a88_85a3_08d3,
    0x1319_8a2e_0370_7344,
    0xa409_3822_299f_31d0,
    0x082e_fa98_ec4e_6c89,
];

struct Case<const BITS: u8> {
    haystack: PackedString<support::Code, BITS>,
    needle: PackedString<support::Code, BITS>,
    haystack_codes: Vec<u8>,
    needle_codes: Vec<u8>,
    expected: bool,
}

fn main() {
    divan::main();
}

fn max_code<const BITS: u8>() -> u8 {
    if BITS == 8 {
        u8::MAX
    } else {
        ((1u16 << BITS) - 1) as u8
    }
}

fn random_codes<const BITS: u8>(len: usize, mut state: u64) -> Vec<u8> {
    let mask = max_code::<BITS>();
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8 & mask
        })
        .collect()
}

fn make_case<const BITS: u8>(case_id: usize, seed: u64) -> Case<BITS> {
    let mut haystack_codes = random_codes::<BITS>(HAYSTACK_LEN, seed ^ 0x5eed_7a11_1234_5678);
    let mut needle_codes = random_codes::<BITS>(NEEDLE_LEN, seed ^ 0xd1ce_ba5e_8765_4321);
    let anchor_len = NEEDLE_LEN.min(64 / usize::from(BITS));

    match case_id {
        0 | 1 => {
            let start = if case_id == 0 {
                0
            } else {
                HAYSTACK_LEN - NEEDLE_LEN
            };
            haystack_codes[start..start + NEEDLE_LEN].copy_from_slice(&needle_codes);
        }
        2 | 3 => {
            // A seeded zero-dominant input makes the intended rare anchor
            // absent: the maximum code is sparse, never a full anchor run.
            haystack_codes.fill(0);
            let stride = anchor_len + 3;
            let offset = (seed as usize) % stride;
            for index in (offset..HAYSTACK_LEN).step_by(stride) {
                haystack_codes[index] = max_code::<BITS>();
            }
            needle_codes.fill(0);
            if case_id == 2 {
                needle_codes[..anchor_len].fill(max_code::<BITS>());
            } else {
                needle_codes[NEEDLE_LEN - anchor_len..].fill(max_code::<BITS>());
            }
        }
        _ => unreachable!(),
    }

    let positions: Vec<_> = haystack_codes
        .windows(needle_codes.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle_codes).then_some(index))
        .collect();
    match case_id {
        0 => assert_eq!(positions.first(), Some(&0), "head-hit fixture is malformed"),
        1 => assert_eq!(
            positions.last(),
            Some(&(HAYSTACK_LEN - NEEDLE_LEN)),
            "tail-hit fixture is malformed"
        ),
        2 | 3 => assert!(positions.is_empty(), "absent fixture is malformed"),
        _ => unreachable!(),
    }
    let expected = !positions.is_empty();

    Case {
        haystack: support::packed::<BITS>(&haystack_codes),
        needle: support::packed::<BITS>(&needle_codes),
        haystack_codes,
        needle_codes,
        expected,
    }
}

fn options(case_id: usize, mode: usize) -> ContainsOptions {
    let (correct_anchor, correct_location) = match case_id {
        0 => (AnchorHint::Unknown, LocationHint::NearStart),
        1 => (AnchorHint::Unknown, LocationHint::NearEnd),
        2 => (AnchorHint::Prefix, LocationHint::Unknown),
        3 => (AnchorHint::Suffix, LocationHint::Unknown),
        _ => unreachable!(),
    };
    let (anchor, location) = match mode {
        0 => (AnchorHint::Unknown, LocationHint::Unknown),
        1 => (correct_anchor, correct_location),
        2 => {
            let wrong_anchor = match correct_anchor {
                AnchorHint::Prefix => AnchorHint::Suffix,
                AnchorHint::Suffix => AnchorHint::Prefix,
                AnchorHint::Unknown => AnchorHint::Unknown,
                _ => AnchorHint::Unknown,
            };
            let wrong_location = match correct_location {
                LocationHint::NearStart => LocationHint::NearEnd,
                LocationHint::NearEnd => LocationHint::NearStart,
                LocationHint::Unknown => LocationHint::Unknown,
                _ => LocationHint::Unknown,
            };
            (wrong_anchor, wrong_location)
        }
        _ => unreachable!(),
    };
    ContainsOptions::default()
        .with_anchor(anchor)
        .with_location(location)
}

fn run<const BITS: u8>(bencher: Bencher, case_id: usize, seed_index: usize, mode: usize) {
    let case = make_case::<BITS>(case_id, SEEDS[seed_index]);
    let expected = case
        .haystack_codes
        .windows(case.needle_codes.len())
        .any(|window| window == case.needle_codes);
    assert_eq!(case.expected, expected);
    for candidate_mode in 0..=2 {
        assert_eq!(
            case.haystack
                .contains_with(&case.needle, options(case_id, candidate_mode)),
            expected,
        );
    }
    let options = options(case_id, mode);
    bencher.bench(|| {
        black_box(&case.haystack).contains_with(black_box(&case.needle), black_box(options))
    });
}

macro_rules! define_width {
    ($module:ident, $bits:literal) => {
        mod $module {
            use super::*;

            macro_rules! define_case {
                ($case:ident, $case_id:literal, $label:literal) => {
                    mod $case {
                        use super::*;

                        #[divan::bench(
                            args = [0usize, 1, 2, 3],
                            name = concat!("search_options/contains/bits_", stringify!($bits), "/", $label, "/default")
                        )]
                        fn default_options(bencher: Bencher, seed_index: usize) {
                            run::<$bits>(bencher, $case_id, seed_index, 0);
                        }

                        #[divan::bench(
                            args = [0usize, 1, 2, 3],
                            name = concat!("search_options/contains/bits_", stringify!($bits), "/", $label, "/correct_hint")
                        )]
                        fn correct_hint(bencher: Bencher, seed_index: usize) {
                            run::<$bits>(bencher, $case_id, seed_index, 1);
                        }

                        #[divan::bench(
                            args = [0usize, 1, 2, 3],
                            name = concat!("search_options/contains/bits_", stringify!($bits), "/", $label, "/wrong_hint")
                        )]
                        fn wrong_hint(bencher: Bencher, seed_index: usize) {
                            run::<$bits>(bencher, $case_id, seed_index, 2);
                        }
                    }
                };
            }

            define_case!(head_hit, 0, "head_hit");
            define_case!(tail_hit, 1, "tail_hit");
            define_case!(rare_prefix_absent, 2, "rare_prefix_absent");
            define_case!(rare_suffix_absent, 3, "rare_suffix_absent");
        }
    };
}

define_width!(bits_4, 4);
define_width!(bits_8, 8);
