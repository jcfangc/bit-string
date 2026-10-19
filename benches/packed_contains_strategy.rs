//! Compare packed `contains` candidate orderings.
//!
//! Run with `cargo bench --features benchmark-internals --bench packed_contains_strategy`.
//! Each scenario aggregates the same 16 input seeds inside each timed iteration.
//! Scenario IDs 0..=3 put one match at the head while varying the backing-view
//! bit phase; 4 and 5 put one match in the middle and at the tail; 6 is random
//! absent; 7 contains repeated matches beginning one quarter into the view;
//! 8..=10 are seeded high-repeat absent cases with a mismatch near the front,
//! middle, and end. All use a 64K character view and a 48-character needle.
//! Compare A (current Any), F (WordsFind First), P (production
//! `find().is_some()`), D (direct character scan), E (two anchors followed by
//! full verification), and G (Any search in bounded word tiles). G is measured
//! with 4, 16, 64, and 256-word tiles. Run native and baseline builds
//! separately; P intentionally follows each build's production `find`
//! dispatch, while the other strategies stay fixed.

use bit_string::PackedString;
use divan::{Bencher, black_box};
use int_intervals::UsizeCO;

#[path = "packed_support/mod.rs"]
mod support;

const HAYSTACK_LEN: usize = 65_536;
const NEEDLE_LEN: usize = 48;
const SEEDS: [u64; 16] = [
    0x243f_6a88_85a3_08d3,
    0x1319_8a2e_0370_7344,
    0xa409_3822_299f_31d0,
    0x082e_fa98_ec4e_6c89,
    0x4528_21e6_38d0_1377,
    0xbe54_66cf_34e9_0c6c,
    0xc0ac_29b7_c97c_50dd,
    0x3f84_d5b5_b547_0917,
    0x9216_d5d9_8979_fb1b,
    0xd131_0ba6_98df_b5ac,
    0x2ffd_72db_d01a_dfb7,
    0xb8e1_afed_6a26_7e96,
    0xba7c_9045_f12c_7f99,
    0x24a1_9947_b391_6cf7,
    0x0801_f2e2_858e_fc16,
    0x6369_20d8_7157_4e69,
];

struct Case<const BITS: u8> {
    haystack: PackedString<support::Code, BITS>,
    needle: PackedString<support::Code, BITS>,
    view_prefix_len: usize,
    expected: bool,
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

fn phase_prefix_len<const BITS: u8>(phase_quartile: usize) -> usize {
    let bits = usize::from(BITS);
    let mut a = 64;
    let mut b = bits;
    while b != 0 {
        (a, b) = (b, a % b);
    }
    let layout_period = 64 / a;
    layout_period * phase_quartile / 4
}

fn make_case<const BITS: u8>(scenario: usize, seed: u64) -> Case<BITS> {
    let mut haystack_codes = random_codes::<BITS>(HAYSTACK_LEN, seed ^ 0x5eed_7a11_1234_5678);
    let needle_codes = if (8..=10).contains(&scenario) {
        let mut codes = vec![0; NEEDLE_LEN];
        let mismatch_index = match scenario {
            8 => 1,
            9 => NEEDLE_LEN / 2,
            10 => NEEDLE_LEN - 2,
            _ => unreachable!(),
        };
        codes[mismatch_index] = max_code::<BITS>();
        codes
    } else {
        random_codes::<BITS>(NEEDLE_LEN, seed ^ 0xd1ce_ba5e_8765_4321)
    };

    let view_prefix_len = if scenario < 4 {
        phase_prefix_len::<BITS>(scenario)
    } else {
        1
    };
    let hit_start = match scenario {
        0..=3 => Some(0),
        4 => Some((HAYSTACK_LEN - NEEDLE_LEN) / 2),
        5 => Some(HAYSTACK_LEN - NEEDLE_LEN),
        6 | 8..=10 => None,
        7 => None,
        _ => unreachable!(),
    };

    if (8..=10).contains(&scenario) {
        // Keep the haystack zero-dominant while varying it by seed. The
        // maximum code appears every 11..=23 characters, so every 48-character
        // window contains multiple mismatches and cannot equal the needle's
        // single-mismatch pattern, including for BITS=1.
        haystack_codes.fill(0);
        let stride = 11 + (seed as usize % 13);
        let offset = (seed.rotate_left(17) as usize) % stride;
        for index in (offset..HAYSTACK_LEN).step_by(stride) {
            haystack_codes[index] = max_code::<BITS>();
        }
    } else if let Some(start) = hit_start {
        haystack_codes[start..start + NEEDLE_LEN].copy_from_slice(&needle_codes);
    } else if scenario == 7 {
        for start in (HAYSTACK_LEN / 4..=HAYSTACK_LEN - NEEDLE_LEN).step_by(NEEDLE_LEN * 4 + 1) {
            haystack_codes[start..start + NEEDLE_LEN].copy_from_slice(&needle_codes);
        }
    } else {
        let mut needle_codes = needle_codes;
        let mut needle_seed = seed ^ 0x8f3f_73b5_cf1c_9ade;
        while haystack_codes
            .windows(NEEDLE_LEN)
            .any(|window| window == needle_codes)
        {
            needle_seed = needle_seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
            needle_codes = random_codes::<BITS>(NEEDLE_LEN, needle_seed);
        }
        let owner = PackedString::<support::Code, BITS>::from_chars(
            core::iter::repeat_n(support::Code(max_code::<BITS>()), view_prefix_len)
                .chain(haystack_codes.iter().copied().map(support::Code)),
        );
        let needle = PackedString::<support::Code, BITS>::from_chars(
            needle_codes.into_iter().map(support::Code),
        );
        return Case {
            haystack: owner,
            needle,
            view_prefix_len,
            expected: false,
        };
    }

    let expected = scenario != 6 && !(8..=10).contains(&scenario);
    let owner = PackedString::<support::Code, BITS>::from_chars(
        core::iter::repeat_n(support::Code(max_code::<BITS>()), view_prefix_len)
            .chain(haystack_codes.into_iter().map(support::Code)),
    );
    let needle = PackedString::<support::Code, BITS>::from_chars(
        needle_codes.into_iter().map(support::Code),
    );
    Case {
        haystack: owner,
        needle,
        view_prefix_len,
        expected,
    }
}

fn run<const BITS: u8>(bencher: Bencher, scenario: usize, strategy: usize) {
    let cases: Vec<_> = SEEDS
        .into_iter()
        .map(|seed| make_case::<BITS>(scenario, seed))
        .collect();
    for case in &cases {
        let haystack = case
            .haystack
            .as_packed_str()
            .slice_from(case.view_prefix_len);
        let needle = case.needle.as_packed_str();
        assert_eq!(haystack.contains(needle), case.expected);
        assert_eq!(
            haystack.find_words_find_for_benchmark(needle).is_some(),
            case.expected
        );
        assert_eq!(haystack.find(needle).is_some(), case.expected);
        assert_eq!(direct_contains(haystack, needle), case.expected);
        assert_eq!(dual_anchor_contains(haystack, needle), case.expected);
        for block_words in [4, 16, 64, 256] {
            assert_eq!(
                haystack.contains_tiled_for_benchmark(needle, block_words),
                case.expected
            );
        }
    }

    bencher.bench(|| {
        let mut found = false;
        for case in &cases {
            let case = black_box(case);
            let haystack = case
                .haystack
                .as_packed_str()
                .slice_from(case.view_prefix_len);
            let needle = case.needle.as_packed_str();
            let result = match strategy {
                0 => haystack.contains(needle),
                1 => haystack.find_words_find_for_benchmark(needle).is_some(),
                2 => haystack.find(needle).is_some(),
                3 => direct_contains(haystack, needle),
                4 => dual_anchor_contains(haystack, needle),
                5..=8 => {
                    haystack.contains_tiled_for_benchmark(needle, [4, 16, 64, 256][strategy - 5])
                }
                _ => unreachable!(),
            };
            found |= black_box(result);
        }
        black_box(found)
    });
}

fn direct_contains<const BITS: u8>(
    haystack: bit_string::PackedStr<'_, support::Code, BITS>,
    needle: bit_string::PackedStr<'_, support::Code, BITS>,
) -> bool {
    let needle_len = needle.char_len();
    let haystack_len = haystack.char_len();
    needle_len <= haystack_len
        && (0..=haystack_len - needle_len).any(|index| haystack.matches_at(index, needle))
}

fn dual_anchor_contains<const BITS: u8>(
    haystack: bit_string::PackedStr<'_, support::Code, BITS>,
    needle: bit_string::PackedStr<'_, support::Code, BITS>,
) -> bool {
    let needle_len = needle.char_len();
    let haystack_len = haystack.char_len();
    if needle_len > haystack_len {
        return false;
    }
    if needle_len == 0 {
        return true;
    }

    let anchor_len = needle_len.min(64 / usize::from(BITS));
    if needle_len <= anchor_len {
        return direct_contains(haystack, needle);
    }
    let suffix_start = needle_len - anchor_len;
    let prefix = needle.slice(UsizeCO::checked_from_start_len(0, anchor_len).unwrap());
    let suffix = needle.slice(UsizeCO::checked_from_start_len(suffix_start, anchor_len).unwrap());
    (0..=haystack_len - needle_len).any(|index| {
        haystack.matches_at(index, prefix)
            && haystack.matches_at(index + suffix_start, suffix)
            && haystack.matches_at(index, needle)
    })
}

macro_rules! define_width_benchmarks {
    ($bits:literal, $tag:ident) => {
        mod $tag {
            use super::*;

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], name = concat!("packed_contains_strategy/bits_", stringify!($bits), "/A_current_any"))]
            fn current_any(bencher: Bencher, scenario: usize) {
                run::<$bits>(bencher, scenario, 0);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], name = concat!("packed_contains_strategy/bits_", stringify!($bits), "/F_words_find_first"))]
            fn words_find_first(bencher: Bencher, scenario: usize) {
                run::<$bits>(bencher, scenario, 1);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], name = concat!("packed_contains_strategy/bits_", stringify!($bits), "/P_production_find"))]
            fn production_find(bencher: Bencher, scenario: usize) {
                run::<$bits>(bencher, scenario, 2);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], name = concat!("packed_contains_strategy/bits_", stringify!($bits), "/D_direct"))]
            fn direct(bencher: Bencher, scenario: usize) {
                run::<$bits>(bencher, scenario, 3);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], name = concat!("packed_contains_strategy/bits_", stringify!($bits), "/E_dual_anchor"))]
            fn dual_anchor(bencher: Bencher, scenario: usize) {
                run::<$bits>(bencher, scenario, 4);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], name = concat!("packed_contains_strategy/bits_", stringify!($bits), "/G_tile_4_words"))]
            fn tile_4_words(bencher: Bencher, scenario: usize) {
                run::<$bits>(bencher, scenario, 5);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], name = concat!("packed_contains_strategy/bits_", stringify!($bits), "/G_tile_16_words"))]
            fn tile_16_words(bencher: Bencher, scenario: usize) {
                run::<$bits>(bencher, scenario, 6);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], name = concat!("packed_contains_strategy/bits_", stringify!($bits), "/G_tile_64_words"))]
            fn tile_64_words(bencher: Bencher, scenario: usize) {
                run::<$bits>(bencher, scenario, 7);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], name = concat!("packed_contains_strategy/bits_", stringify!($bits), "/G_tile_256_words"))]
            fn tile_256_words(bencher: Bencher, scenario: usize) {
                run::<$bits>(bencher, scenario, 8);
            }
        }
    };
}

fn main() {
    divan::main();
}

define_width_benchmarks!(1, bits_1);
define_width_benchmarks!(2, bits_2);
define_width_benchmarks!(3, bits_3);
define_width_benchmarks!(4, bits_4);
define_width_benchmarks!(5, bits_5);
define_width_benchmarks!(6, bits_6);
define_width_benchmarks!(7, bits_7);
define_width_benchmarks!(8, bits_8);
