#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use int_intervals::UsizeCO;
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

fn get_sequential_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let mut source = Vec::with_capacity(len + 1);
    source.push(0);
    source.extend_from_slice(&input);
    let owner = packed::<BITS>(&source);
    let value = owner.as_packed_str().slice_from(1);
    bencher.bench(|| {
        let value = black_box(value);
        let mut checksum = 0u8;
        for index in 0..len {
            checksum ^= value.get(index).unwrap().0;
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

fn get_random_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let mut source = Vec::with_capacity(len + 1);
    source.push(0);
    source.extend_from_slice(&input);
    let owner = packed::<BITS>(&source);
    let value = owner.as_packed_str().slice_from(1);
    let indexes = indices(len);
    bencher.bench(|| {
        let value = black_box(value);
        let indexes = black_box(&indexes);
        let mut checksum = 0u8;
        for &index in indexes {
            checksum ^= value.get(index).unwrap().0;
        }
        black_box(checksum)
    });
}

fn slice_packed_string<const BITS: u8>(bencher: Bencher, len: usize) {
    let value = packed::<BITS>(&codes(BITS, len));
    bencher.bench(|| black_box(value.slice(slice_interval(len / 4, len / 2))));
}

fn slice_packed_str<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let mut source = Vec::with_capacity(len + 1);
    source.push(0);
    source.extend_from_slice(&input);
    let owner = packed::<BITS>(&source);
    let value = owner.as_packed_str().slice_from(1);
    bencher.bench(|| black_box(value.slice(slice_interval(len / 4, len / 2))));
}

fn slice_interval(start: usize, len: usize) -> UsizeCO {
    UsizeCO::checked_from_start_len(start, len).unwrap()
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
                                                                    "/ours_packed_str_unaligned"
                                                                )
                                                            )]
            fn ours_packed_str(bencher: Bencher) {
                super::get_sequential_packed_str::<$bits>(bencher, $len);
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
                                                                    "packed_access/get_random_stream/",
                                                                    stringify!($case),
                                                                    "/ours_packed_string"
                                                                )
                                                            )]
            fn ours_packed_string_random(bencher: Bencher) {
                super::get_random_packed::<$bits>(bencher, $len);
            }

            #[divan::bench(
                                                                name = concat!(
                                                                    "packed_access/get_random_stream/",
                                                                    stringify!($case),
                                                                    "/ours_packed_str_unaligned"
                                                                )
                                                            )]
            fn ours_packed_str_random(bencher: Bencher) {
                super::get_random_packed_str::<$bits>(bencher, $len);
            }

            #[divan::bench(
                                                                name = concat!(
                                                                    "packed_access/get_random_stream/",
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

crate::for_each_packed_access_bench_case!(define_case);

mod slice_cases {
    use super::*;

    macro_rules! define_case {
        ($case:ident, $bits:literal, $len:literal) => {
            mod $case {
                use super::*;

                #[divan::bench(name = concat!("packed_slice/slice_middle/unaligned_view/", stringify!($case), "/ours_packed_string_aligned"))]
                fn ours_packed_string(bencher: Bencher) {
                    super::super::slice_packed_string::<$bits>(bencher, $len);
                }

                #[divan::bench(name = concat!("packed_slice/slice_middle/unaligned_view/", stringify!($case), "/ours_packed_str_unaligned"))]
                fn ours_packed_str(bencher: Bencher) {
                    super::super::slice_packed_str::<$bits>(bencher, $len);
                }
            }
        };
    }

    crate::for_each_packed_access_bench_case!(define_case);
}
