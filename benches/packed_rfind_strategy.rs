// BITS=8 is intentionally omitted: its candidate-scan A/B was measured before
// selecting the character-level fallback, so the production method and naive
// reference are now the same algorithm. BITS=7 keeps the live comparison.

use bit_string::PackedString;
use divan::{Bencher, black_box};

#[path = "packed_support/mod.rs"]
mod support;

fn build_case<
    const BITS: u8,
    const HAYSTACK_LEN: usize,
    const NEEDLE_LEN: usize,
    const POSITION: u8,
>() -> (
    PackedString<support::Code, BITS>,
    PackedString<support::Code, BITS>,
    Option<usize>,
) {
    let max_code = if BITS == 8 {
        u8::MAX
    } else {
        ((1u16 << BITS) - 1) as u8
    };
    let needle_codes = vec![max_code; NEEDLE_LEN];
    let start = match POSITION {
        0 => 0,
        1 => (HAYSTACK_LEN - NEEDLE_LEN) / 2,
        2 => HAYSTACK_LEN - NEEDLE_LEN,
        3 => HAYSTACK_LEN,
        _ => unreachable!(),
    };
    let mut haystack_codes = vec![0; HAYSTACK_LEN];
    if POSITION != 3 {
        haystack_codes[start..start + NEEDLE_LEN].copy_from_slice(&needle_codes);
    }

    (
        support::packed::<BITS>(&haystack_codes),
        support::packed::<BITS>(&needle_codes),
        (POSITION != 3).then_some(start),
    )
}

fn naive_rfind<const BITS: u8>(
    haystack: &PackedString<support::Code, BITS>,
    needle: &PackedString<support::Code, BITS>,
) -> Option<usize> {
    (0..=haystack.char_len().saturating_sub(needle.char_len()))
        .rev()
        .find(|&index| haystack.matches_at(index, needle))
}

fn benchmark<
    const BITS: u8,
    const HAYSTACK_LEN: usize,
    const NEEDLE_LEN: usize,
    const POSITION: u8,
>(
    bencher: Bencher,
    use_library_rfind: bool,
) {
    let (haystack, needle, expected) = build_case::<BITS, HAYSTACK_LEN, NEEDLE_LEN, POSITION>();
    assert_eq!(haystack.rfind(&needle), expected);
    assert_eq!(naive_rfind(&haystack, &needle), expected);

    bencher.bench(|| {
        black_box(if use_library_rfind {
            haystack.rfind(&needle)
        } else {
            naive_rfind(&haystack, &needle)
        })
    });
}

macro_rules! define_case {
    ($name:ident, $bits:literal, $haystack_len:literal, $needle_len:literal, $position:literal, $width:literal, $where_:expr) => {
        mod $name {
            #[divan::bench(name = concat!("packed_rfind_strategy/", $width, "/", $where_, "/library_rfind"))]
            fn library_rfind(bencher: divan::Bencher) {
                crate::benchmark::<$bits, $haystack_len, $needle_len, $position>(bencher, true);
            }

            #[divan::bench(name = concat!("packed_rfind_strategy/", $width, "/", $where_, "/naive_reference"))]
            fn naive_reference(bencher: divan::Bencher) {
                crate::benchmark::<$bits, $haystack_len, $needle_len, $position>(bencher, false);
            }
        }
    };
}

macro_rules! define_width_cases {
    ($module:ident, $bits:literal, $haystack_len:literal, $length:literal, $width:literal) => {
        mod $module {
            define_case!(
                short_head,
                $bits,
                $haystack_len,
                8,
                0,
                $width,
                concat!($length, "/short/head")
            );
            define_case!(
                short_middle,
                $bits,
                $haystack_len,
                8,
                1,
                $width,
                concat!($length, "/short/middle")
            );
            define_case!(
                short_tail,
                $bits,
                $haystack_len,
                8,
                2,
                $width,
                concat!($length, "/short/tail")
            );
            define_case!(
                short_absent,
                $bits,
                $haystack_len,
                8,
                3,
                $width,
                concat!($length, "/short/absent")
            );
            define_case!(
                long_head,
                $bits,
                $haystack_len,
                24,
                0,
                $width,
                concat!($length, "/long/head")
            );
            define_case!(
                long_middle,
                $bits,
                $haystack_len,
                24,
                1,
                $width,
                concat!($length, "/long/middle")
            );
            define_case!(
                long_tail,
                $bits,
                $haystack_len,
                24,
                2,
                $width,
                concat!($length, "/long/tail")
            );
            define_case!(
                long_absent,
                $bits,
                $haystack_len,
                24,
                3,
                $width,
                concat!($length, "/long/absent")
            );
        }
    };
}

define_width_cases!(bits_7_len_1024, 7, 1_024, "len_1024", "bits_7");
define_width_cases!(bits_7_len_65536, 7, 65_536, "len_65536", "bits_7");
define_width_cases!(bits_7_len_1048576, 7, 1_048_576, "len_1048576", "bits_7");

fn main() {
    divan::main();
}
