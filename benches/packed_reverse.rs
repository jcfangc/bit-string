#![allow(dead_code)]

#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{codes, packed};

fn main() {
    divan::main();
}

fn reverse_assign<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.reverse_assign();
        black_box(&*value);
        black_box(value.char_len())
    });
}

macro_rules! define_reverse_case {
    ($case:ident, $bits:literal, $len:literal) => {
        #[divan::bench(name = concat!("packed_reverse/reverse_assign/", stringify!($case), "/ours_packed_string"))]
        fn $case(bencher: Bencher) {
            reverse_assign::<$bits>(bencher, $len);
        }
    };
}

#[cfg(not(feature = "characterization"))]
macro_rules! define_default_cases {
    () => {
        define_reverse_case!(bits_1_len_64, 1, 64);
        define_reverse_case!(bits_1_len_65536, 1, 65_536);
        define_reverse_case!(bits_2_len_64, 2, 64);
        define_reverse_case!(bits_2_len_65536, 2, 65_536);
        define_reverse_case!(bits_3_len_64, 3, 64);
        define_reverse_case!(bits_3_len_65536, 3, 65_536);
        define_reverse_case!(bits_4_len_64, 4, 64);
        define_reverse_case!(bits_4_len_65536, 4, 65_536);
        define_reverse_case!(bits_5_len_64, 5, 64);
        define_reverse_case!(bits_5_len_65536, 5, 65_536);
        define_reverse_case!(bits_6_len_64, 6, 64);
        define_reverse_case!(bits_6_len_65536, 6, 65_536);
        define_reverse_case!(bits_7_len_64, 7, 64);
        define_reverse_case!(bits_7_len_65536, 7, 65_536);
        define_reverse_case!(bits_8_len_64, 8, 64);
        define_reverse_case!(bits_8_len_65536, 8, 65_536);
    };
}

#[cfg(not(feature = "characterization"))]
define_default_cases!();

#[cfg(feature = "characterization")]
macro_rules! define_characterization_cases {
    () => {
        define_reverse_case!(bits_1_len_1, 1, 1);
        define_reverse_case!(bits_1_len_7, 1, 7);
        define_reverse_case!(bits_1_len_8, 1, 8);
        define_reverse_case!(bits_1_len_21, 1, 21);
        define_reverse_case!(bits_1_len_22, 1, 22);
        define_reverse_case!(bits_1_len_63, 1, 63);
        define_reverse_case!(bits_1_len_64, 1, 64);
        define_reverse_case!(bits_1_len_65, 1, 65);
        define_reverse_case!(bits_1_len_1024, 1, 1024);
        define_reverse_case!(bits_1_len_65536, 1, 65536);
        define_reverse_case!(bits_2_len_1, 2, 1);
        define_reverse_case!(bits_2_len_7, 2, 7);
        define_reverse_case!(bits_2_len_8, 2, 8);
        define_reverse_case!(bits_2_len_21, 2, 21);
        define_reverse_case!(bits_2_len_22, 2, 22);
        define_reverse_case!(bits_2_len_31, 2, 31);
        define_reverse_case!(bits_2_len_32, 2, 32);
        define_reverse_case!(bits_2_len_33, 2, 33);
        define_reverse_case!(bits_2_len_63, 2, 63);
        define_reverse_case!(bits_2_len_64, 2, 64);
        define_reverse_case!(bits_2_len_65, 2, 65);
        define_reverse_case!(bits_2_len_1024, 2, 1024);
        define_reverse_case!(bits_2_len_65536, 2, 65536);
        define_reverse_case!(bits_3_len_1, 3, 1);
        define_reverse_case!(bits_3_len_7, 3, 7);
        define_reverse_case!(bits_3_len_8, 3, 8);
        define_reverse_case!(bits_3_len_21, 3, 21);
        define_reverse_case!(bits_3_len_22, 3, 22);
        define_reverse_case!(bits_3_len_20, 3, 20);
        define_reverse_case!(bits_3_len_63, 3, 63);
        define_reverse_case!(bits_3_len_64, 3, 64);
        define_reverse_case!(bits_3_len_65, 3, 65);
        define_reverse_case!(bits_3_len_1024, 3, 1024);
        define_reverse_case!(bits_3_len_65536, 3, 65536);
        define_reverse_case!(bits_4_len_1, 4, 1);
        define_reverse_case!(bits_4_len_7, 4, 7);
        define_reverse_case!(bits_4_len_8, 4, 8);
        define_reverse_case!(bits_4_len_21, 4, 21);
        define_reverse_case!(bits_4_len_22, 4, 22);
        define_reverse_case!(bits_4_len_15, 4, 15);
        define_reverse_case!(bits_4_len_16, 4, 16);
        define_reverse_case!(bits_4_len_17, 4, 17);
        define_reverse_case!(bits_4_len_63, 4, 63);
        define_reverse_case!(bits_4_len_64, 4, 64);
        define_reverse_case!(bits_4_len_65, 4, 65);
        define_reverse_case!(bits_4_len_1024, 4, 1024);
        define_reverse_case!(bits_4_len_65536, 4, 65536);
        define_reverse_case!(bits_5_len_1, 5, 1);
        define_reverse_case!(bits_5_len_7, 5, 7);
        define_reverse_case!(bits_5_len_8, 5, 8);
        define_reverse_case!(bits_5_len_21, 5, 21);
        define_reverse_case!(bits_5_len_22, 5, 22);
        define_reverse_case!(bits_5_len_11, 5, 11);
        define_reverse_case!(bits_5_len_12, 5, 12);
        define_reverse_case!(bits_5_len_13, 5, 13);
        define_reverse_case!(bits_5_len_63, 5, 63);
        define_reverse_case!(bits_5_len_64, 5, 64);
        define_reverse_case!(bits_5_len_65, 5, 65);
        define_reverse_case!(bits_5_len_1024, 5, 1024);
        define_reverse_case!(bits_5_len_65536, 5, 65536);
        define_reverse_case!(bits_6_len_1, 6, 1);
        define_reverse_case!(bits_6_len_7, 6, 7);
        define_reverse_case!(bits_6_len_8, 6, 8);
        define_reverse_case!(bits_6_len_21, 6, 21);
        define_reverse_case!(bits_6_len_22, 6, 22);
        define_reverse_case!(bits_6_len_9, 6, 9);
        define_reverse_case!(bits_6_len_10, 6, 10);
        define_reverse_case!(bits_6_len_11, 6, 11);
        define_reverse_case!(bits_6_len_63, 6, 63);
        define_reverse_case!(bits_6_len_64, 6, 64);
        define_reverse_case!(bits_6_len_65, 6, 65);
        define_reverse_case!(bits_6_len_1024, 6, 1024);
        define_reverse_case!(bits_6_len_65536, 6, 65536);
        define_reverse_case!(bits_7_len_1, 7, 1);
        define_reverse_case!(bits_7_len_7, 7, 7);
        define_reverse_case!(bits_7_len_8, 7, 8);
        define_reverse_case!(bits_7_len_21, 7, 21);
        define_reverse_case!(bits_7_len_22, 7, 22);
        define_reverse_case!(bits_7_len_9, 7, 9);
        define_reverse_case!(bits_7_len_10, 7, 10);
        define_reverse_case!(bits_7_len_63, 7, 63);
        define_reverse_case!(bits_7_len_64, 7, 64);
        define_reverse_case!(bits_7_len_65, 7, 65);
        define_reverse_case!(bits_7_len_1024, 7, 1024);
        define_reverse_case!(bits_7_len_65536, 7, 65536);
        define_reverse_case!(bits_8_len_1, 8, 1);
        define_reverse_case!(bits_8_len_7, 8, 7);
        define_reverse_case!(bits_8_len_8, 8, 8);
        define_reverse_case!(bits_8_len_21, 8, 21);
        define_reverse_case!(bits_8_len_22, 8, 22);
        define_reverse_case!(bits_8_len_9, 8, 9);
        define_reverse_case!(bits_8_len_63, 8, 63);
        define_reverse_case!(bits_8_len_64, 8, 64);
        define_reverse_case!(bits_8_len_65, 8, 65);
        define_reverse_case!(bits_8_len_1024, 8, 1024);
        define_reverse_case!(bits_8_len_65536, 8, 65536);
    };
}

#[cfg(feature = "characterization")]
define_characterization_cases!();
