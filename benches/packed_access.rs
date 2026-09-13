#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{codes, indices, packed};

fn main() {
    divan::main();
}

fn get_sequential_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    bencher.bench(|| {
        let value = black_box(&value);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= value.get(index).unwrap().0;
        }
        black_box(checksum)
    });
}

fn get_sequential_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| {
        let input = black_box(&input);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= input[index];
        }
        black_box(checksum)
    });
}

fn get_random_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    let indexes = indices(len);
    bencher.bench(|| {
        let value = black_box(&value);
        let indexes = black_box(&indexes);
        let mut checksum = 0u8;
        for &index in indexes {
            checksum ^= value.get(index).unwrap().0;
        }
        black_box(checksum)
    });
}

fn get_random_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let indexes = indices(len);
    bencher.bench(|| {
        let input = black_box(&input);
        let indexes = black_box(&indexes);
        let mut checksum = 0u8;
        for &index in indexes {
            checksum ^= input[index];
        }
        black_box(checksum)
    });
}

macro_rules! define_case {
    ($case:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(
                                                                name = concat!(
                                                                    "packed_access/get_sequential/",
                                                                    stringify!($case),
                                                                    "/ours_packed_string"
                                                                )
                                                            )]
            fn ours_packed_string(bencher: Bencher) {
                super::get_sequential_packed::<$bits>(bencher, $len);
            }

            #[divan::bench(
                                                                name = concat!(
                                                                    "packed_access/get_sequential/",
                                                                    stringify!($case),
                                                                    "/vec_u8"
                                                                )
                                                            )]
            fn vec_u8(bencher: Bencher) {
                super::get_sequential_vec::<$bits>(bencher, $len);
            }

            #[divan::bench(
                                                                name = concat!(
                                                                    "packed_access/get_random/",
                                                                    stringify!($case),
                                                                    "/ours_packed_string"
                                                                )
                                                            )]
            fn ours_packed_string_random(bencher: Bencher) {
                super::get_random_packed::<$bits>(bencher, $len);
            }

            #[divan::bench(
                                                                name = concat!(
                                                                    "packed_access/get_random/",
                                                                    stringify!($case),
                                                                    "/vec_u8"
                                                                )
                                                            )]
            fn vec_u8_random(bencher: Bencher) {
                super::get_random_vec::<$bits>(bencher, $len);
            }
        }
    };
}

crate::for_each_packed_bench_case!(define_case);
