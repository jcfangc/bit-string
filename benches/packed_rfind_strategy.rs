// Compare the current bit-level WordsFind path with character-aligned reverse
// scans using a prefix or suffix anchor before full match verification.

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

fn build_case<
    const BITS: u8,
    const HAYSTACK_LEN: usize,
    const NEEDLE_LEN: usize,
    const POSITION: u8,
    const PROFILE: u8,
>() -> (
    PackedString<support::Code, BITS>,
    PackedString<support::Code, BITS>,
    Option<usize>,
) {
    let anchor_len = NEEDLE_LEN.min(64 / usize::from(BITS));
    let mut haystack_codes = if PROFILE == 2 {
        random_codes::<BITS>(HAYSTACK_LEN, 0x5eed_7a11_1234_5678)
    } else {
        vec![0; HAYSTACK_LEN]
    };
    let mut needle_codes = if PROFILE == 2 {
        random_codes::<BITS>(NEEDLE_LEN, 0xd1ce_ba5e_8765_4321)
    } else {
        vec![0; NEEDLE_LEN]
    };
    match PROFILE {
        // The prefix is common (zero); only the suffix anchor is selective.
        0 => needle_codes[NEEDLE_LEN - anchor_len..].fill(max_code::<BITS>()),
        // The prefix anchor is selective; the suffix is common (zero).
        1 => needle_codes[..anchor_len].fill(max_code::<BITS>()),
        // Random data exercises a broad code distribution. For the absent
        // case, try a bounded number of generated needles and skip the case
        // if this finite code domain does not provide an absent value.
        2 => {
            if POSITION == 3 {
                let mut seed = 0xd1ce_ba5e_8765_4321_u64;
                for _ in 0..64 {
                    if !haystack_codes
                        .windows(NEEDLE_LEN)
                        .any(|window| window == needle_codes)
                    {
                        break;
                    }
                    seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
                    needle_codes = random_codes::<BITS>(NEEDLE_LEN, seed);
                }
            }
        }
        // A repeated zero haystack with a late needle mismatch creates many
        // long-prefix candidates without a full match.
        3 => needle_codes[(anchor_len + 2).min(NEEDLE_LEN - 1)] = 1,
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
        .rposition(|window| window == needle_codes);

    let mut source_codes = Vec::with_capacity(HAYSTACK_LEN + 1);
    source_codes.push(max_code::<BITS>());
    source_codes.extend(haystack_codes);

    (
        support::packed::<BITS>(&source_codes),
        support::packed::<BITS>(&needle_codes),
        expected,
    )
}

fn aligned_rfind<const BITS: u8>(
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

    (0..=latest_start).rev().find(|&candidate| {
        let anchor_matches = haystack.matches_at(candidate + anchor_start, anchor);
        anchor_matches && (anchor_len == needle_len || haystack.matches_at(candidate, needle))
    })
}

fn direct_rfind<const BITS: u8>(
    haystack: &PackedString<support::Code, BITS>,
    needle: &PackedString<support::Code, BITS>,
) -> Option<usize> {
    let haystack = haystack.as_packed_str().slice_from(1);
    let needle = needle.as_packed_str();
    let needle_len = needle.char_len();
    if needle_len > haystack.char_len() {
        return None;
    }

    (0..=haystack.char_len() - needle_len)
        .rev()
        .find(|&candidate| haystack.matches_at(candidate, needle))
}

fn dual_anchor_rfind<const BITS: u8>(
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
            .rev()
            .find(|&candidate| haystack.matches_at(candidate, needle));
    }

    let suffix_start = needle_len - anchor_len;
    let prefix = needle.slice(interval(0, anchor_len));
    let suffix = needle.slice(interval(suffix_start, anchor_len));
    (0..=haystack.char_len() - needle_len)
        .rev()
        .find(|&candidate| {
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
) {
    let (haystack, needle, expected) =
        build_case::<BITS, HAYSTACK_LEN, NEEDLE_LEN, POSITION, PROFILE>();
    if PROFILE == 2 && POSITION == 3 && expected.is_some() {
        return;
    }
    if PROFILE == 2 && POSITION < 3 {
        let intended_start = match POSITION {
            0 => 0,
            1 => (HAYSTACK_LEN - NEEDLE_LEN) / 2,
            2 => HAYSTACK_LEN - NEEDLE_LEN,
            _ => unreachable!(),
        };
        if expected != Some(intended_start) {
            return;
        }
    }
    let actual = match method {
        0 => haystack
            .as_packed_str()
            .slice_from(1)
            .rfind(needle.as_packed_str()),
        1 => aligned_rfind(&haystack, &needle, false),
        2 => aligned_rfind(&haystack, &needle, true),
        3 => direct_rfind(&haystack, &needle),
        4 => dual_anchor_rfind(&haystack, &needle),
        _ => unreachable!(),
    };
    assert_eq!(actual, expected);

    bencher.bench(|| {
        black_box(match method {
            0 => haystack
                .as_packed_str()
                .slice_from(1)
                .rfind(needle.as_packed_str()),
            1 => aligned_rfind(&haystack, &needle, false),
            2 => aligned_rfind(&haystack, &needle, true),
            3 => direct_rfind(&haystack, &needle),
            4 => dual_anchor_rfind(&haystack, &needle),
            _ => unreachable!(),
        })
    });
}

macro_rules! define_case {
    ($module:ident, $bits:literal, $haystack_len:literal, $needle_len:literal,
     $position:literal, $profile:literal, $label_prefix:literal, $label_suffix:literal) => {
        mod $module {
            #[divan::bench(name = concat!("packed_rfind_strategy/", $label_prefix, $label_suffix, "/words_find_prefix"))]
            fn words_find_prefix(bencher: divan::Bencher) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 0);
            }

            #[divan::bench(name = concat!("packed_rfind_strategy/", $label_prefix, $label_suffix, "/aligned_prefix"))]
            fn aligned_prefix(bencher: divan::Bencher) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 1);
            }

            #[divan::bench(name = concat!("packed_rfind_strategy/", $label_prefix, $label_suffix, "/aligned_suffix"))]
            fn aligned_suffix(bencher: divan::Bencher) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 2);
            }

            #[divan::bench(name = concat!("packed_rfind_strategy/", $label_prefix, $label_suffix, "/direct_matches_at"))]
            fn direct_matches_at(bencher: divan::Bencher) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 3);
            }

            #[divan::bench(name = concat!("packed_rfind_strategy/", $label_prefix, $label_suffix, "/dual_anchor"))]
            fn dual_anchor(bencher: divan::Bencher) {
                crate::run::<$bits, $haystack_len, $needle_len, $position, $profile>(bencher, 4);
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
                long_high_repeat_absent,
                $bits,
                $len,
                24,
                3,
                3,
                $len_name,
                "/long/high_repeat/absent"
            );
        }
    };
}

macro_rules! define_high_repeat_matrix {
    ($prefix:ident, $bits:literal, $len:literal, $len_name:literal) => {
        mod $prefix {
            define_case!(
                long_high_repeat_absent,
                $bits,
                $len,
                24,
                3,
                3,
                $len_name,
                "/long/high_repeat/absent"
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

fn main() {
    divan::main();
}
