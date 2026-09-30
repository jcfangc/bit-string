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
