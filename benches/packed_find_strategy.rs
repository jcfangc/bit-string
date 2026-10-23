// Compare the current bit-level WordsFind path with character-aligned forward
// scans using a prefix or suffix anchor before full match verification.
// Run with `cargo bench --features benchmark-internals --bench packed_find_strategy`.
//
// Workload-score policy for reports based on this matrix:
// 1. Use the mean timing over the independent seed arguments within a cell.
// 2. Within each data profile, average valid haystack-length, needle-length,
//    and result-position cells equally; do not weight profiles by row count.
// 3. Average the available data-profile means equally for each BITS.
// Missing cells are conditional omissions: a seed is retained only when its
// observed find result matches the requested position. In particular, short
// random needles over small code alphabets may not support absent/head/middle
// outcomes, and prefix-selectivity profiles are indistinguishable when the
// <=64-bit anchor spans the whole needle. Reports must list these omissions.
// The two haystack lengths are equally weighted workload classes; times remain
// absolute operation latency and are not normalized by character count.
// The two mirrored high-repeat mismatch placements are equally weighted within
// that profile: one mismatch lies in the prefix anchor, the other in the suffix.
// For confidence intervals, first aggregate paired strategy timings by seed
// using the policy above, then bootstrap seed IDs (not Divan timer samples).

use bit_string::PackedString;
use divan::{Bencher, black_box};
use int_intervals::UsizeCO;

#[path = "packed_support/mod.rs"]
mod support;

fn interval(start: usize, len: usize) -> UsizeCO {
    UsizeCO::checked_from_start_len(start, len).unwrap()
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

fn biased_codes<const BITS: u8>(len: usize, mut state: u64, common_code: u8) -> Vec<u8> {
    let mask = max_code::<BITS>();
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            if state & 7 == 0 {
                state as u8 & mask
            } else {
                common_code
            }
        })
        .collect()
}

fn biased_codes_without_max<const BITS: u8>(
    len: usize,
    mut state: u64,
    common_code: u8,
) -> Vec<u8> {
    let max_code = max_code::<BITS>();
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            if state & 7 == 0 {
                (state as u8) % max_code
            } else {
                common_code
            }
        })
        .collect()
}

// Keep a stable, independent input sample for each seed. Divan runs each seed
// as a separate argument so input-to-input variation can be analyzed apart
// from repeated timing samples.
const INPUT_SEEDS: [u64; 16] = [
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

fn build_case<
    const BITS: u8,
    const HAYSTACK_LEN: usize,
    const NEEDLE_LEN: usize,
    const POSITION: u8,
    const PROFILE: u8,
>(
    seed: u64,
) -> (
    PackedString<support::Code, BITS>,
    PackedString<support::Code, BITS>,
    Option<usize>,
) {
    let anchor_len = NEEDLE_LEN.min(64 / usize::from(BITS));
    // Both prefix-selectivity profiles use the same zero-dominant haystack;
    // they differ only in whether the needle's first anchor is common or rare.
    let common_code = 0;
    let mut haystack_codes = if PROFILE == 2 {
        random_codes::<BITS>(HAYSTACK_LEN, seed ^ 0x5eed_7a11_1234_5678)
    } else if PROFILE >= 3 {
        // Reserve the maximum code for the needle's mismatch. This guarantees
        // both mirrored high-repeat absent cases are actually absent.
        biased_codes_without_max::<BITS>(HAYSTACK_LEN, seed ^ 0x5eed_7a11_1234_5678, common_code)
    } else {
        biased_codes::<BITS>(HAYSTACK_LEN, seed ^ 0x5eed_7a11_1234_5678, common_code)
    };
    let mut needle_codes = if PROFILE >= 3 {
        vec![0; NEEDLE_LEN]
    } else {
        random_codes::<BITS>(NEEDLE_LEN, seed ^ 0xd1ce_ba5e_8765_4321)
    };
    let rare_suffix_len = (NEEDLE_LEN - anchor_len).min(4);
    match PROFILE {
        // Keep the prefix anchor common (zero) and place rare values in the
        // non-overlapping suffix so both anchors remain distinguishable.
        0 => {
            needle_codes[..anchor_len].fill(0);
            needle_codes[NEEDLE_LEN - rare_suffix_len..].fill(max_code::<BITS>());
        }
        // Keep the suffix anchor common (zero) and place rare values before it.
        1 => {
            needle_codes[NEEDLE_LEN - anchor_len..].fill(0);
            needle_codes[..rare_suffix_len].fill(max_code::<BITS>());
        }
        // Random data exercises a broad code distribution. For the absent
        // case, try a bounded number of generated needles and skip the case
        // if this finite code domain does not provide an absent value.
        2 => {
            if POSITION == 3 {
                let mut needle_seed = seed ^ 0xd1ce_ba5e_8765_4321_u64;
                for _ in 0..64 {
                    if !haystack_codes
                        .windows(NEEDLE_LEN)
                        .any(|window| window == needle_codes)
                    {
                        break;
                    }
                    needle_seed = needle_seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
                    needle_codes = random_codes::<BITS>(NEEDLE_LEN, needle_seed);
                }
            }
        }
        // Put a late mismatch in the suffix-only region, outside the prefix
        // anchor, so only the suffix anchor rejects it.
        3 => {
            let suffix_start = NEEDLE_LEN - anchor_len;
            let exclusive_len = anchor_len.min(suffix_start);
            let mismatch_offset = seed as usize % exclusive_len;
            let suffix_only_start = anchor_len.max(suffix_start);
            needle_codes[suffix_only_start + mismatch_offset] = max_code::<BITS>();
        }
        // Mirror the late mismatch around the needle midpoint, in the
        // prefix-only region, so only the prefix anchor rejects it.
        4 => {
            let suffix_start = NEEDLE_LEN - anchor_len;
            let exclusive_len = anchor_len.min(suffix_start);
            let mismatch_offset = seed as usize % exclusive_len;
            needle_codes[exclusive_len - 1 - mismatch_offset] = max_code::<BITS>();
        }
        _ => unreachable!(),
    }

    let start = match POSITION {
        0 => 0,
        1 => (HAYSTACK_LEN - NEEDLE_LEN) / 2,
        2 => HAYSTACK_LEN - NEEDLE_LEN,
        3 => HAYSTACK_LEN,
        _ => unreachable!(),
    };
    if POSITION != 3 {
        haystack_codes[start..start + NEEDLE_LEN].copy_from_slice(&needle_codes);
    }
    let expected = haystack_codes
        .windows(NEEDLE_LEN)
        .position(|window| window == needle_codes);

    let mut source_codes = Vec::with_capacity(HAYSTACK_LEN + 1);
    source_codes.push(max_code::<BITS>());
    source_codes.extend(haystack_codes);

    (
        support::packed::<BITS>(&source_codes),
        support::packed::<BITS>(&needle_codes),
        expected,
    )
}

fn aligned_find<const BITS: u8>(
    haystack: &PackedString<support::Code, BITS>,
    needle: &PackedString<support::Code, BITS>,
    use_suffix: bool,
) -> Option<usize> {
    // Start at character 1 so both widths exercise a character-aligned,
    // word-unaligned PackedStr view (bit offsets 7 and 8 respectively).
    let haystack = haystack.as_packed_str().slice_from(1);
    let needle = needle.as_packed_str();
    let needle_len = needle.char_len();
    if needle_len > haystack.char_len() {
        return None;
    }

    let anchor_len = needle_len.min(64 / usize::from(BITS));
    let anchor_start = if use_suffix {
        needle_len - anchor_len
    } else {
        0
    };
    let anchor = needle.slice(interval(anchor_start, anchor_len));
    let latest_start = haystack.char_len() - needle_len;

    (0..=latest_start).find(|&candidate| {
        let anchor_matches = haystack.matches_at(candidate + anchor_start, anchor);
        anchor_matches && (anchor_len == needle_len || haystack.matches_at(candidate, needle))
    })
}

fn direct_find<const BITS: u8>(
    haystack: &PackedString<support::Code, BITS>,
    needle: &PackedString<support::Code, BITS>,
) -> Option<usize> {
    let haystack = haystack.as_packed_str().slice_from(1);
    let needle = needle.as_packed_str();
    let needle_len = needle.char_len();
    if needle_len > haystack.char_len() {
        return None;
    }

    (0..=haystack.char_len() - needle_len).find(|&candidate| haystack.matches_at(candidate, needle))
}

fn dual_anchor_find<const BITS: u8>(
    haystack: &PackedString<support::Code, BITS>,
    needle: &PackedString<support::Code, BITS>,
) -> Option<usize> {
    let haystack = haystack.as_packed_str().slice_from(1);
    let needle = needle.as_packed_str();
    let needle_len = needle.char_len();
    if needle_len > haystack.char_len() {
        return None;
    }

    let anchor_len = needle_len.min(64 / usize::from(BITS));
    if anchor_len == needle_len {
        return (0..=haystack.char_len() - needle_len)
            .find(|&candidate| haystack.matches_at(candidate, needle));
    }

    let suffix_start = needle_len - anchor_len;
    let prefix = needle.slice(interval(0, anchor_len));
    let suffix = needle.slice(interval(suffix_start, anchor_len));
    (0..=haystack.char_len() - needle_len).find(|&candidate| {
        haystack.matches_at(candidate, prefix)
            && haystack.matches_at(candidate + suffix_start, suffix)
            && haystack.matches_at(candidate, needle)
    })
}

fn run<
    const BITS: u8,
    const HAYSTACK_LEN: usize,
    const NEEDLE_LEN: usize,
    const POSITION: u8,
    const PROFILE: u8,
>(
    bencher: Bencher,
    method: u8,
    seed_index: usize,
) {
    let seed = INPUT_SEEDS[seed_index];
    let (haystack, needle, expected) =
        build_case::<BITS, HAYSTACK_LEN, NEEDLE_LEN, POSITION, PROFILE>(seed);
    let intended_match = match POSITION {
        0 => Some(0),
        1 => Some((HAYSTACK_LEN - NEEDLE_LEN) / 2),
        2 => Some(HAYSTACK_LEN - NEEDLE_LEN),
        3 => None,
        _ => unreachable!(),
    };
    // Random finite alphabets can make some requested outcomes unavailable.
    // Omit that input seed identically for all strategies rather than relabel it.
    if expected != intended_match {
        return;
    }

    let actual = match method {
        0 => haystack
            .as_packed_str()
            .slice_from(1)
            .find_words_find_for_benchmark(needle.as_packed_str()),
        1 => aligned_find(&haystack, &needle, false),
        2 => aligned_find(&haystack, &needle, true),
        3 => direct_find(&haystack, &needle),
        4 => dual_anchor_find(&haystack, &needle),
        _ => unreachable!(),
    };
    assert_eq!(actual, expected);

    bencher.bench(|| {
        let haystack = black_box(&haystack);
        let needle = black_box(&needle);
        black_box(match method {
            0 => haystack
                .as_packed_str()
                .slice_from(1)
                .find_words_find_for_benchmark(needle.as_packed_str()),
            1 => aligned_find(&haystack, &needle, false),
            2 => aligned_find(&haystack, &needle, true),
            3 => direct_find(&haystack, &needle),
            4 => dual_anchor_find(&haystack, &needle),
            _ => unreachable!(),
        })
    });
}

macro_rules! define_case {
    ($module:ident, $bits:literal, $haystack_len:literal, $needle_len:literal,
     $position:literal, $profile:literal, $label_prefix:literal, $label_suffix:literal) => {
        mod $module {
            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], name = concat!("packed_find_strategy/", $label_prefix, $label_suffix, "/words_find_prefix"))]
            fn words_find_prefix(bencher: divan::Bencher, seed_index: usize) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 0, seed_index);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], name = concat!("packed_find_strategy/", $label_prefix, $label_suffix, "/aligned_prefix"))]
            fn aligned_prefix(bencher: divan::Bencher, seed_index: usize) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 1, seed_index);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], name = concat!("packed_find_strategy/", $label_prefix, $label_suffix, "/aligned_suffix"))]
            fn aligned_suffix(bencher: divan::Bencher, seed_index: usize) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 2, seed_index);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], name = concat!("packed_find_strategy/", $label_prefix, $label_suffix, "/direct_matches_at"))]
            fn direct_matches_at(bencher: divan::Bencher, seed_index: usize) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 3, seed_index);
            }

            #[divan::bench(args = [0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], name = concat!("packed_find_strategy/", $label_prefix, $label_suffix, "/dual_anchor"))]
            fn dual_anchor(bencher: divan::Bencher, seed_index: usize) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 4, seed_index);
            }
        }
    };
}

macro_rules! define_matrix {
    ($prefix:ident, $bits:literal, $len:literal, $len_name:literal) => {
        mod $prefix {
            // At eight codes the <=64-bit anchor is the entire short needle,
            // so prefix/suffix selectivity profiles are indistinguishable.
            define_case!(short_head, $bits, $len, 8, 0, 0, $len_name, "/short/head");
            define_case!(
                short_middle,
                $bits,
                $len,
                8,
                1,
                0,
                $len_name,
                "/short/middle"
            );
            define_case!(short_tail, $bits, $len, 8, 2, 0, $len_name, "/short/tail");
            define_case!(
                short_absent,
                $bits,
                $len,
                8,
                3,
                0,
                $len_name,
                "/short/absent"
            );
            define_case!(
                long_profile_0_head,
                $bits,
                $len,
                24,
                0,
                0,
                $len_name,
                "/long/prefix_common/head"
            );
            define_case!(
                long_profile_0_middle,
                $bits,
                $len,
                24,
                1,
                0,
                $len_name,
                "/long/prefix_common/middle"
            );
            define_case!(
                long_profile_0_tail,
                $bits,
                $len,
                24,
                2,
                0,
                $len_name,
                "/long/prefix_common/tail"
            );
            define_case!(
                long_profile_0_absent,
                $bits,
                $len,
                24,
                3,
                0,
                $len_name,
                "/long/prefix_common/absent"
            );
            define_case!(
                long_profile_1_head,
                $bits,
                $len,
                24,
                0,
                1,
                $len_name,
                "/long/prefix_rare/head"
            );
            define_case!(
                long_profile_1_middle,
                $bits,
                $len,
                24,
                1,
                1,
                $len_name,
                "/long/prefix_rare/middle"
            );
            define_case!(
                long_profile_1_tail,
                $bits,
                $len,
                24,
                2,
                1,
                $len_name,
                "/long/prefix_rare/tail"
            );
            define_case!(
                long_profile_1_absent,
                $bits,
                $len,
                24,
                3,
                1,
                $len_name,
                "/long/prefix_rare/absent"
            );
            define_case!(
                short_random_head,
                $bits,
                $len,
                8,
                0,
                2,
                $len_name,
                "/short/random/head"
            );
            define_case!(
                short_random_middle,
                $bits,
                $len,
                8,
                1,
                2,
                $len_name,
                "/short/random/middle"
            );
            define_case!(
                short_random_tail,
                $bits,
                $len,
                8,
                2,
                2,
                $len_name,
                "/short/random/tail"
            );
            define_case!(
                long_random_head,
                $bits,
                $len,
                24,
                0,
                2,
                $len_name,
                "/long/random/head"
            );
            define_case!(
                long_random_middle,
                $bits,
                $len,
                24,
                1,
                2,
                $len_name,
                "/long/random/middle"
            );
            define_case!(
                long_random_tail,
                $bits,
                $len,
                24,
                2,
                2,
                $len_name,
                "/long/random/tail"
            );
            define_case!(
                long_random_absent,
                $bits,
                $len,
                24,
                3,
                2,
                $len_name,
                "/long/random/absent"
            );
            define_case!(
                long_high_repeat_late_mismatch_absent,
                $bits,
                $len,
                24,
                3,
                3,
                $len_name,
                "/long/high_repeat_late_mismatch/absent"
            );
            define_case!(
                long_high_repeat_early_mismatch_absent,
                $bits,
                $len,
                24,
                3,
                4,
                $len_name,
                "/long/high_repeat_early_mismatch/absent"
            );
        }
    };
}

macro_rules! define_high_repeat_matrix {
    ($prefix:ident, $bits:literal, $len:literal, $len_name:literal) => {
        mod $prefix {
            define_case!(
                long_high_repeat_late_mismatch_absent,
                $bits,
                $len,
                24,
                3,
                3,
                $len_name,
                "/long/high_repeat_late_mismatch/absent"
            );
            define_case!(
                long_high_repeat_early_mismatch_absent,
                $bits,
                $len,
                24,
                3,
                4,
                $len_name,
                "/long/high_repeat_early_mismatch/absent"
            );
        }
    };
}

// BITS=1/2 have an anchor spanning the entire 24-code needle, so they cannot
// exercise a late mismatch beyond the anchor. Keep this case for widths where
// the long needle extends past its 64-bit anchor.
define_high_repeat_matrix!(bits_3_high_repeat_65536, 3, 65_536, "bits_3/len_65536");
define_high_repeat_matrix!(
    bits_3_high_repeat_1048576,
    3,
    1_048_576,
    "bits_3/len_1048576"
);
define_high_repeat_matrix!(bits_4_high_repeat_65536, 4, 65_536, "bits_4/len_65536");
define_high_repeat_matrix!(
    bits_4_high_repeat_1048576,
    4,
    1_048_576,
    "bits_4/len_1048576"
);
define_high_repeat_matrix!(bits_5_high_repeat_65536, 5, 65_536, "bits_5/len_65536");
define_high_repeat_matrix!(
    bits_5_high_repeat_1048576,
    5,
    1_048_576,
    "bits_5/len_1048576"
);
define_high_repeat_matrix!(bits_6_high_repeat_65536, 6, 65_536, "bits_6/len_65536");
define_high_repeat_matrix!(
    bits_6_high_repeat_1048576,
    6,
    1_048_576,
    "bits_6/len_1048576"
);

macro_rules! define_short_random_absent_case {
    ($prefix:ident, $bits:literal, $len:literal, $len_name:literal) => {
        mod $prefix {
            define_case!(
                short_random_absent,
                $bits,
                $len,
                8,
                3,
                2,
                $len_name,
                "/short/random/absent"
            );
        }
    };
}

// For BITS=1, an 8-code needle has only 256 possible values; a 65K random
// haystack is overwhelmingly likely to contain all of them. For BITS=2, the
// 1M haystack similarly covers almost all 8-code patterns. Those absent cases
// are omitted because they cannot be constructed reliably from random data.
define_short_random_absent_case!(bits_2_short_absent_65536, 2, 65_536, "bits_2/len_65536");
define_short_random_absent_case!(bits_3_short_absent_65536, 3, 65_536, "bits_3/len_65536");
define_short_random_absent_case!(
    bits_3_short_absent_1048576,
    3,
    1_048_576,
    "bits_3/len_1048576"
);
define_short_random_absent_case!(bits_4_short_absent_65536, 4, 65_536, "bits_4/len_65536");
define_short_random_absent_case!(
    bits_4_short_absent_1048576,
    4,
    1_048_576,
    "bits_4/len_1048576"
);
define_short_random_absent_case!(bits_5_short_absent_65536, 5, 65_536, "bits_5/len_65536");
define_short_random_absent_case!(
    bits_5_short_absent_1048576,
    5,
    1_048_576,
    "bits_5/len_1048576"
);
define_short_random_absent_case!(bits_6_short_absent_65536, 6, 65_536, "bits_6/len_65536");
define_short_random_absent_case!(
    bits_6_short_absent_1048576,
    6,
    1_048_576,
    "bits_6/len_1048576"
);
define_short_random_absent_case!(bits_7_short_absent_65536, 7, 65_536, "bits_7/len_65536");
define_short_random_absent_case!(
    bits_7_short_absent_1048576,
    7,
    1_048_576,
    "bits_7/len_1048576"
);
define_short_random_absent_case!(bits_8_short_absent_65536, 8, 65_536, "bits_8/len_65536");
define_short_random_absent_case!(
    bits_8_short_absent_1048576,
    8,
    1_048_576,
    "bits_8/len_1048576"
);

define_matrix!(bits_7_len_65536, 7, 65_536, "bits_7/len_65536");
define_matrix!(bits_7_len_1048576, 7, 1_048_576, "bits_7/len_1048576");
define_matrix!(bits_8_len_65536, 8, 65_536, "bits_8/len_65536");
define_matrix!(bits_8_len_1048576, 8, 1_048_576, "bits_8/len_1048576");

macro_rules! define_extended_boundary_matrix {
    ($prefix:ident, $bits:literal, $len:literal, $len_name:literal) => {
        mod $prefix {
            define_case!(
                short_random_head,
                $bits,
                $len,
                8,
                0,
                2,
                $len_name,
                "/short/random/head"
            );
            define_case!(
                short_random_middle,
                $bits,
                $len,
                8,
                1,
                2,
                $len_name,
                "/short/random/middle"
            );
            define_case!(
                short_random_tail,
                $bits,
                $len,
                8,
                2,
                2,
                $len_name,
                "/short/random/tail"
            );
            define_case!(
                long_random_head,
                $bits,
                $len,
                24,
                0,
                2,
                $len_name,
                "/long/random/head"
            );
            define_case!(
                long_random_middle,
                $bits,
                $len,
                24,
                1,
                2,
                $len_name,
                "/long/random/middle"
            );
            define_case!(
                long_random_tail,
                $bits,
                $len,
                24,
                2,
                2,
                $len_name,
                "/long/random/tail"
            );
            define_case!(
                long_random_absent,
                $bits,
                $len,
                24,
                3,
                2,
                $len_name,
                "/long/random/absent"
            );
        }
    };
}

macro_rules! define_prefix_profile_matrix {
    ($prefix:ident, $bits:literal, $len:literal, $len_name:literal) => {
        mod $prefix {
            define_case!(
                head_common,
                $bits,
                $len,
                24,
                0,
                0,
                $len_name,
                "/long/prefix_common/head"
            );
            define_case!(
                middle_common,
                $bits,
                $len,
                24,
                1,
                0,
                $len_name,
                "/long/prefix_common/middle"
            );
            define_case!(
                tail_common,
                $bits,
                $len,
                24,
                2,
                0,
                $len_name,
                "/long/prefix_common/tail"
            );
            define_case!(
                absent_common,
                $bits,
                $len,
                24,
                3,
                0,
                $len_name,
                "/long/prefix_common/absent"
            );
            define_case!(
                head_rare,
                $bits,
                $len,
                24,
                0,
                1,
                $len_name,
                "/long/prefix_rare/head"
            );
            define_case!(
                middle_rare,
                $bits,
                $len,
                24,
                1,
                1,
                $len_name,
                "/long/prefix_rare/middle"
            );
            define_case!(
                tail_rare,
                $bits,
                $len,
                24,
                2,
                1,
                $len_name,
                "/long/prefix_rare/tail"
            );
            define_case!(
                absent_rare,
                $bits,
                $len,
                24,
                3,
                1,
                $len_name,
                "/long/prefix_rare/absent"
            );
        }
    };
}

define_extended_boundary_matrix!(bits_1_len_65536, 1, 65_536, "bits_1/len_65536");
define_extended_boundary_matrix!(bits_1_len_1048576, 1, 1_048_576, "bits_1/len_1048576");
define_extended_boundary_matrix!(bits_2_len_65536, 2, 65_536, "bits_2/len_65536");
define_extended_boundary_matrix!(bits_2_len_1048576, 2, 1_048_576, "bits_2/len_1048576");
define_extended_boundary_matrix!(bits_3_len_65536, 3, 65_536, "bits_3/len_65536");
define_extended_boundary_matrix!(bits_3_len_1048576, 3, 1_048_576, "bits_3/len_1048576");
define_extended_boundary_matrix!(bits_4_len_65536, 4, 65_536, "bits_4/len_65536");
define_extended_boundary_matrix!(bits_4_len_1048576, 4, 1_048_576, "bits_4/len_1048576");
define_extended_boundary_matrix!(bits_5_len_65536, 5, 65_536, "bits_5/len_65536");
define_extended_boundary_matrix!(bits_5_len_1048576, 5, 1_048_576, "bits_5/len_1048576");
define_extended_boundary_matrix!(bits_6_len_65536, 6, 65_536, "bits_6/len_65536");
define_extended_boundary_matrix!(bits_6_len_1048576, 6, 1_048_576, "bits_6/len_1048576");

// Prefix-common/rare are distinct only when the 64-bit anchor is shorter than
// the needle. For an 8-code needle this distinction disappears at BITS=1/2.
define_prefix_profile_matrix!(bits_3_prefix_65536, 3, 65_536, "bits_3/len_65536");
define_prefix_profile_matrix!(bits_3_prefix_1048576, 3, 1_048_576, "bits_3/len_1048576");
define_prefix_profile_matrix!(bits_4_prefix_65536, 4, 65_536, "bits_4/len_65536");
define_prefix_profile_matrix!(bits_4_prefix_1048576, 4, 1_048_576, "bits_4/len_1048576");
define_prefix_profile_matrix!(bits_5_prefix_65536, 5, 65_536, "bits_5/len_65536");
define_prefix_profile_matrix!(bits_5_prefix_1048576, 5, 1_048_576, "bits_5/len_1048576");
define_prefix_profile_matrix!(bits_6_prefix_65536, 6, 65_536, "bits_6/len_65536");
define_prefix_profile_matrix!(bits_6_prefix_1048576, 6, 1_048_576, "bits_6/len_1048576");

fn main() {
    divan::main();
}
