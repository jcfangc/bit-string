#[path = "packed_support/mod.rs"]
mod support;

use bit_string::PackedString;
use divan::{Bencher, black_box};
use support::{codes, packed};

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum KeepRate {
    None,
    Half,
    All,
}

impl KeepRate {
    fn keeps(self, code: u8) -> bool {
        match self {
            Self::None => false,
            Self::Half => code & 1 == 0,
            Self::All => true,
        }
    }
}

fn retain_in_place<const BITS: u8>(bencher: Bencher, len: usize, keep_rate: KeepRate) {
    let input = packed::<BITS>(&codes(BITS, len));
    assert_retain_matches_reference(&input, keep_rate);

    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        black_box(&mut *value).retain(|character| keep_rate.keeps(black_box(character.0)));
        black_box(&*value);
        black_box(value.char_len());
    });
}

fn retain_rebuild_reference<const BITS: u8>(bencher: Bencher, len: usize, keep_rate: KeepRate) {
    let input = packed::<BITS>(&codes(BITS, len));
    assert_retain_matches_reference(&input, keep_rate);

    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let value = black_box(value);
        let retained = value
            .iter()
            .filter(|&character| keep_rate.keeps(black_box(character.0)));
        *value = PackedString::from_chars(retained);
        black_box(&*value);
        black_box(value.char_len());
    });
}

fn assert_retain_matches_reference<const BITS: u8>(
    input: &PackedString<support::Code, BITS>,
    keep_rate: KeepRate,
) {
    let expected = input
        .iter()
        .filter(|&character| keep_rate.keeps(character.0))
        .map(|character| character.0)
        .collect::<Vec<_>>();
    let mut actual = input.clone();
    actual.retain(|character| keep_rate.keeps(character.0));

    assert_eq!(
        actual
            .iter()
            .map(|character| character.0)
            .collect::<Vec<_>>(),
        expected
    );
}

macro_rules! define_case {
    ($case:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            macro_rules! define_keep_rate {
                ($scenario:literal, $name:ident, $reference_name:ident, $rate:expr) => {
                    #[divan::bench(name = concat!("packed_retain/", $scenario, "/", stringify!($case), "/ours_packed_string_in_place"))]
                    fn $name(bencher: Bencher) {
                        super::retain_in_place::<$bits>(bencher, $len, $rate);
                    }

                    #[divan::bench(name = concat!("packed_retain/", $scenario, "/", stringify!($case), "/ours_packed_string_rebuild_reference"))]
                    fn $reference_name(bencher: Bencher) {
                        super::retain_rebuild_reference::<$bits>(bencher, $len, $rate);
                    }
                };
            }

            define_keep_rate!("keep_none", keep_none, keep_none_reference, KeepRate::None);
            define_keep_rate!("keep_half", keep_half, keep_half_reference, KeepRate::Half);
            define_keep_rate!("keep_all", keep_all, keep_all_reference, KeepRate::All);
        }
    };
}

crate::for_each_packed_bench_case!(define_case);
