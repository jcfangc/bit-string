#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{codes, packed};

fn main() {
    divan::main();
}

fn collect_via_iter<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    bencher.bench(|| black_box(&value).iter().collect::<Vec<_>>());
}

fn convert_to_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    bencher.bench(|| black_box(&value).to_vec());
}

macro_rules! define_case {
    ($case:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(name = concat!("packed_conversion/to_vec/", stringify!($case), "/iter_collect"))]
            fn iter_collect(bencher: Bencher) {
                super::collect_via_iter::<$bits>(bencher, $len);
            }

            #[divan::bench(name = concat!("packed_conversion/to_vec/", stringify!($case), "/to_vec"))]
            fn to_vec(bencher: Bencher) {
                super::convert_to_vec::<$bits>(bencher, $len);
            }
        }
    };
}

crate::for_each_packed_bench_case!(define_case);

macro_rules! define_boundary_case {
    ($case:ident, $width:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(name = concat!("packed_conversion/to_vec_boundary/", stringify!($width), "/", stringify!($case), "/iter_collect"))]
            fn iter_collect(bencher: Bencher) {
                super::super::collect_via_iter::<$bits>(bencher, $len);
            }

            #[divan::bench(name = concat!("packed_conversion/to_vec_boundary/", stringify!($width), "/", stringify!($case), "/to_vec"))]
            fn to_vec(bencher: Bencher) {
                super::super::convert_to_vec::<$bits>(bencher, $len);
            }
        }
    };
}

macro_rules! define_width_boundaries {
    ($width:ident, $bits:literal; $($case:ident: $len:literal),+ $(,)?) => {
        mod $width {
            use super::*;

            $(define_boundary_case!($case, $width, $bits, $len);)+
        }
    };
}

mod boundaries {
    use super::*;

    define_width_boundaries!(bits_1, 1;
        len_16: 16, len_31: 31, len_32: 32, len_63: 63, len_64: 64,
        len_65: 65, len_95: 95, len_127: 127, len_128: 128, len_129: 129,
    );
    define_width_boundaries!(bits_2, 2;
        len_16: 16, len_31: 31, len_32: 32, len_63: 63, len_64: 64,
        len_65: 65, len_95: 95, len_127: 127, len_128: 128, len_129: 129,
    );
    define_width_boundaries!(bits_3, 3;
        len_16: 16, len_31: 31, len_32: 32, len_63: 63, len_64: 64,
        len_65: 65, len_95: 95, len_127: 127, len_128: 128, len_129: 129,
    );
    define_width_boundaries!(bits_4, 4;
        len_16: 16, len_31: 31, len_32: 32, len_63: 63, len_64: 64,
        len_65: 65, len_95: 95, len_127: 127, len_128: 128, len_129: 129,
    );
    define_width_boundaries!(bits_5, 5;
        len_16: 16, len_31: 31, len_32: 32, len_63: 63, len_64: 64,
        len_65: 65, len_95: 95, len_127: 127, len_128: 128, len_129: 129,
    );
    define_width_boundaries!(bits_6, 6;
        len_16: 16, len_31: 31, len_32: 32, len_63: 63, len_64: 64,
        len_65: 65, len_95: 95, len_127: 127, len_128: 128, len_129: 129,
    );
    define_width_boundaries!(bits_7, 7;
        len_16: 16, len_31: 31, len_32: 32, len_63: 63, len_64: 64,
        len_65: 65, len_95: 95, len_127: 127, len_128: 128, len_129: 129,
    );
    define_width_boundaries!(bits_8, 8;
        len_16: 16, len_31: 31, len_32: 32, len_63: 63, len_64: 64,
        len_65: 65, len_95: 95, len_127: 127, len_128: 128, len_129: 129,
    );
}
