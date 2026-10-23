//! Paired measurements for bit-string `contains` location hints.
//!
//! Cases place a single match at the first or last valid bit offset, or use a
//! high-repeat absent input with a rare prefix or suffix. Each benchmark
//! compares default options, the correct hint, the opposite hint, and a direct
//! `matches_at_str` reference over the same seeded inputs. Fixture construction
//! and oracle checks are outside the timed closure.

use bit_string::{AnchorHint, BitString, ContainsOptions, LocationHint};
use divan::{Bencher, black_box};
use int_intervals::UsizeCO;

const HAYSTACK_LEN: usize = 65_536;
const NEEDLE_LEN: usize = 128;
const SEEDS: [u64; 4] = [
    0x243f_6a88_85a3_08d3,
    0x1319_8a2e_0370_7344,
    0xa409_3822_299f_31d0,
    0x082e_fa98_ec4e_6c89,
];

struct Case {
    haystack: BitString,
    needle: BitString,
    haystack_bits: Vec<bool>,
    needle_bits: Vec<bool>,
    expected: bool,
}

fn main() {
    divan::main();
}

fn random_bits(len: usize, mut state: u64) -> Vec<bool> {
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state & 1 != 0
        })
        .collect()
}

fn make_case(case_id: usize, seed: u64) -> Case {
    let mut haystack_bits = random_bits(HAYSTACK_LEN, seed ^ 0x5eed_7a11_1234_5678);
    let mut needle_bits = random_bits(NEEDLE_LEN, seed ^ 0xd1ce_ba5e_8765_4321);
    match case_id {
        0 | 1 => {
            let start = if case_id == 0 {
                0
            } else {
                HAYSTACK_LEN - NEEDLE_LEN
            };
            haystack_bits[start..start + NEEDLE_LEN].copy_from_slice(&needle_bits);
        }
        2 | 3 => {
            let common_bit = seed & 1 != 0;
            let rare_bit = !common_bit;
            haystack_bits.fill(common_bit);
            needle_bits.fill(common_bit);
            if case_id == 2 {
                needle_bits[..64].fill(rare_bit);
            } else {
                needle_bits[NEEDLE_LEN - 64..].fill(rare_bit);
            }
        }
        _ => unreachable!(),
    }

    let positions: Vec<_> = haystack_bits
        .windows(NEEDLE_LEN)
        .enumerate()
        .filter_map(|(index, window)| (window == needle_bits).then_some(index))
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

    Case {
        haystack: haystack_bits.iter().copied().collect(),
        needle: needle_bits.iter().copied().collect(),
        haystack_bits,
        needle_bits,
        expected: case_id < 2,
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
        2 => (
            match correct_anchor {
                AnchorHint::Prefix => AnchorHint::Suffix,
                AnchorHint::Suffix => AnchorHint::Prefix,
                _ => AnchorHint::Unknown,
            },
            match correct_location {
                LocationHint::NearStart => LocationHint::NearEnd,
                LocationHint::NearEnd => LocationHint::NearStart,
                _ => LocationHint::Unknown,
            },
        ),
        _ => (bit_string::AnchorHint::Unknown, LocationHint::Unknown),
    };
    ContainsOptions::default()
        .with_anchor(anchor)
        .with_location(location)
}

fn direct_reference(haystack: &BitString, needle: &BitString) -> bool {
    let haystack = haystack.as_bit_str();
    let needle = needle.as_bit_str();
    needle.bit_len() <= haystack.bit_len()
        && (0..=haystack.bit_len() - needle.bit_len())
            .any(|index| haystack.matches_at_str(index, needle))
}

fn anchored_reference(haystack: &BitString, needle: &BitString, suffix: bool) -> bool {
    let haystack = haystack.as_bit_str();
    let needle = needle.as_bit_str();
    if needle.bit_len() > haystack.bit_len() {
        return false;
    }

    let anchor_len = needle.bit_len().min(64);
    let anchor_start = if suffix {
        needle.bit_len() - anchor_len
    } else {
        0
    };
    let anchor = needle.slice(
        UsizeCO::checked_from_start_len(anchor_start, anchor_len)
            .expect("anchor interval is in bounds"),
    );
    (0..=haystack.bit_len() - needle.bit_len()).any(|start| {
        haystack.matches_at_str(start + anchor_start, anchor)
            && haystack.matches_at_str(start, needle)
    })
}

fn run(bencher: Bencher, case_id: usize, seed_index: usize, mode: usize) {
    let case = make_case(case_id, SEEDS[seed_index]);
    let expected = case
        .haystack_bits
        .windows(case.needle_bits.len())
        .any(|window| window == case.needle_bits);
    assert_eq!(case.expected, expected);
    assert_eq!(direct_reference(&case.haystack, &case.needle), expected);
    assert_eq!(
        anchored_reference(&case.haystack, &case.needle, false),
        expected,
    );
    assert_eq!(
        anchored_reference(&case.haystack, &case.needle, true),
        expected,
    );
    for candidate_mode in 0..=2 {
        assert_eq!(
            case.haystack
                .as_bit_str()
                .contains_str_with(case.needle.as_bit_str(), options(case_id, candidate_mode)),
            expected,
        );
    }

    let options = (mode <= 2).then(|| options(case_id, mode));
    bencher.bench(|| {
        let haystack = black_box(&case.haystack);
        let needle = black_box(&case.needle);
        match mode {
            0..=2 => haystack
                .as_bit_str()
                .contains_str_with(needle.as_bit_str(), black_box(options.unwrap())),
            3 => direct_reference(haystack, needle),
            4 => anchored_reference(haystack, needle, false),
            5 => anchored_reference(haystack, needle, true),
            _ => unreachable!(),
        }
    });
}

macro_rules! define_case {
    ($module:ident, $case_id:literal, $label:literal) => {
        mod $module {
            use super::*;

            #[divan::bench(
                        args = [0usize, 1, 2, 3],
                        name = concat!("search_options/bit_contains/", $label, "/default")
                    )]
            fn default_options(bencher: Bencher, seed_index: usize) {
                run(bencher, $case_id, seed_index, 0);
            }

            #[divan::bench(
                        args = [0usize, 1, 2, 3],
                        name = concat!("search_options/bit_contains/", $label, "/correct_hint")
                    )]
            fn correct_hint(bencher: Bencher, seed_index: usize) {
                run(bencher, $case_id, seed_index, 1);
            }

            #[divan::bench(
                        args = [0usize, 1, 2, 3],
                        name = concat!("search_options/bit_contains/", $label, "/wrong_hint")
                    )]
            fn wrong_hint(bencher: Bencher, seed_index: usize) {
                run(bencher, $case_id, seed_index, 2);
            }

            #[divan::bench(
                        args = [0usize, 1, 2, 3],
                        name = concat!("search_options/bit_contains/", $label, "/direct_reference")
                    )]
            fn direct_reference(bencher: Bencher, seed_index: usize) {
                run(bencher, $case_id, seed_index, 3);
            }

            #[divan::bench(
                        args = [0usize, 1, 2, 3],
                        name = concat!("search_options/bit_contains/", $label, "/prefix_anchor")
                    )]
            fn prefix_anchor(bencher: Bencher, seed_index: usize) {
                run(bencher, $case_id, seed_index, 4);
            }

            #[divan::bench(
                        args = [0usize, 1, 2, 3],
                        name = concat!("search_options/bit_contains/", $label, "/suffix_anchor")
                    )]
            fn suffix_anchor(bencher: Bencher, seed_index: usize) {
                run(bencher, $case_id, seed_index, 5);
            }
        }
    };
}

define_case!(head_hit, 0, "head_hit");
define_case!(tail_hit, 1, "tail_hit");
define_case!(rare_prefix_absent, 2, "rare_prefix_absent");
define_case!(rare_suffix_absent, 3, "rare_suffix_absent");
