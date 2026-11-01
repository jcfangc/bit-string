use bit_string::BitString;
use divan::{Bencher, black_box};
use int_intervals::UsizeCO;

fn main() {
    divan::main();
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Pattern {
    Zero,
    One,
    Alternating,
    Random,
}

fn bit_at(index: usize, pattern: Pattern) -> bool {
    match pattern {
        Pattern::Zero => false,
        Pattern::One => true,
        Pattern::Alternating => index % 2 == 0,
        Pattern::Random => {
            let mut value = (index as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            (value ^ (value >> 31)) >> 63 != 0
        }
    }
}

fn make_bit_string(len: usize, pattern: Pattern) -> BitString {
    (0..len).map(|index| bit_at(index, pattern)).collect()
}

fn make_view_backing(len: usize, offset: usize, pattern: Pattern) -> BitString {
    (0..offset + len)
        .map(|index| bit_at(index, pattern))
        .collect()
}

fn make_view(bits: &BitString, len: usize, offset: usize) -> bit_string::BitStr<'_> {
    if len == 0 {
        return bits.as_bit_str().slice_from(offset);
    }

    bits.as_bit_str()
        .slice(UsizeCO::try_new(offset, offset + len).unwrap())
}

fn bench_owned_api(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);
    let expected: Vec<_> = (0..len).map(|index| bit_at(index, pattern)).collect();
    assert_eq!(bits.to_bool_vec(), expected);
    bencher.bench(|| black_box(&bits).to_bool_vec());
}

fn bench_owned_reference(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);
    let expected: Vec<_> = (0..len).map(|index| bit_at(index, pattern)).collect();
    assert_eq!(bits.iter().collect::<Vec<_>>(), expected);
    bencher.bench(|| black_box(&bits).iter().collect::<Vec<_>>());
}

fn bench_view_api(bencher: Bencher, len: usize, offset: usize, pattern: Pattern) {
    let backing = make_view_backing(len, offset, pattern);
    let view = make_view(&backing, len, offset);
    let expected: Vec<_> = (offset..offset + len)
        .map(|index| bit_at(index, pattern))
        .collect();
    assert_eq!(view.to_bool_vec(), expected);
    bencher.bench(|| black_box(&view).to_bool_vec());
}

fn bench_view_reference(bencher: Bencher, len: usize, offset: usize, pattern: Pattern) {
    let backing = make_view_backing(len, offset, pattern);
    let view = make_view(&backing, len, offset);
    let expected: Vec<_> = (offset..offset + len)
        .map(|index| bit_at(index, pattern))
        .collect();
    assert_eq!(view.iter().collect::<Vec<_>>(), expected);
    bencher.bench(|| black_box(&view).iter().collect::<Vec<_>>());
}

macro_rules! define_case_with_views {
    ($case:ident, $len:literal, $length_label:literal, $pattern:ident, $pattern_label:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(name = concat!("to_bool_vec/len_", $length_label, "/", $pattern_label, "/owned/ours_bit"))]
            fn owned_api(bencher: Bencher) {
                super::bench_owned_api(bencher, $len, Pattern::$pattern);
            }

            #[divan::bench(name = concat!("to_bool_vec/len_", $length_label, "/", $pattern_label, "/owned/iter_collect_reference"))]
            fn owned_reference(bencher: Bencher) {
                super::bench_owned_reference(bencher, $len, Pattern::$pattern);
            }

            view_case!(offset_1, 1, "1", $len, $length_label, $pattern, $pattern_label);
            view_case!(offset_7, 7, "7", $len, $length_label, $pattern, $pattern_label);
            view_case!(offset_63, 63, "63", $len, $length_label, $pattern, $pattern_label);
            view_case!(offset_64, 64, "64", $len, $length_label, $pattern, $pattern_label);
        }
    };
}

macro_rules! view_case {
    ($module:ident, $offset:literal, $offset_label:literal, $len:literal, $length_label:literal, $pattern:ident, $pattern_label:literal) => {
        mod $module {
            use super::*;

            #[divan::bench(name = concat!("to_bool_vec/len_", $length_label, "/", $pattern_label, "/offset_", $offset_label, "/ours_bit"))]
            fn api(bencher: Bencher) {
                super::super::bench_view_api(bencher, $len, $offset, Pattern::$pattern);
            }

            #[divan::bench(name = concat!("to_bool_vec/len_", $length_label, "/", $pattern_label, "/offset_", $offset_label, "/iter_collect_reference"))]
            fn reference(bencher: Bencher) {
                super::super::bench_view_reference(bencher, $len, $offset, Pattern::$pattern);
            }
        }
    };
}

// Compact regression set; characterization expands to the full length matrix.
#[cfg(not(feature = "characterization"))]
define_case_with_views!(len_65_random, 65, "65", Random, "random");
#[cfg(not(feature = "characterization"))]
define_case_with_views!(len_65536_random, 65_536, "65536", Random, "random");
#[cfg(not(feature = "characterization"))]
define_case_with_views!(len_65536_zero, 65_536, "65536", Zero, "zero");

#[cfg(feature = "characterization")]
define_case_with_views!(len_0_zero, 0, "0", Zero, "zero");
#[cfg(feature = "characterization")]
define_case_with_views!(len_1_random, 1, "1", Random, "random");
#[cfg(feature = "characterization")]
define_case_with_views!(len_63_alternating, 63, "63", Alternating, "alternating");
#[cfg(feature = "characterization")]
define_case_with_views!(len_64_random, 64, "64", Random, "random");
#[cfg(feature = "characterization")]
define_case_with_views!(len_65_zero, 65, "65", Zero, "zero");
#[cfg(feature = "characterization")]
define_case_with_views!(len_65_one, 65, "65", One, "one");
#[cfg(feature = "characterization")]
define_case_with_views!(len_65_alternating, 65, "65", Alternating, "alternating");
#[cfg(feature = "characterization")]
define_case_with_views!(len_65_random_full, 65, "65", Random, "random_full");
#[cfg(feature = "characterization")]
define_case_with_views!(len_4096_zero, 4_096, "4096", Zero, "zero");
#[cfg(feature = "characterization")]
define_case_with_views!(len_4096_one, 4_096, "4096", One, "one");
#[cfg(feature = "characterization")]
define_case_with_views!(
    len_4096_alternating,
    4_096,
    "4096",
    Alternating,
    "alternating"
);
#[cfg(feature = "characterization")]
define_case_with_views!(len_4096_random, 4_096, "4096", Random, "random");
#[cfg(feature = "characterization")]
define_case_with_views!(len_65536_one, 65_536, "65536", One, "one");
#[cfg(feature = "characterization")]
define_case_with_views!(
    len_65536_alternating,
    65_536,
    "65536",
    Alternating,
    "alternating"
);
#[cfg(feature = "characterization")]
define_case_with_views!(
    len_65536_random_full,
    65_536,
    "65536",
    Random,
    "random_full"
);
#[cfg(feature = "characterization")]
define_case_with_views!(len_1048576_zero, 1_048_576, "1048576", Zero, "zero");
#[cfg(feature = "characterization")]
define_case_with_views!(len_1048576_one, 1_048_576, "1048576", One, "one");
#[cfg(feature = "characterization")]
define_case_with_views!(
    len_1048576_alternating,
    1_048_576,
    "1048576",
    Alternating,
    "alternating"
);
#[cfg(feature = "characterization")]
define_case_with_views!(len_1048576_random, 1_048_576, "1048576", Random, "random");
