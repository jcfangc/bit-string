#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{codes, packed};

fn main() {
    divan::main();
}

fn iterate_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let value = packed::<BITS>(&input);
    bencher.bench(|| {
        let value = black_box(&value);
        let checksum = value.iter().fold(0u8, |checksum, code| checksum ^ code.0);
        black_box(checksum)
    });
}

fn iterate_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| {
        let input = black_box(&input);
        let checksum = input.iter().fold(0u8, |checksum, &code| checksum ^ code);
        black_box(checksum)
    });
}

macro_rules! define_case {
    ($case:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(
                                                                        name = concat!(
                                    "packed_iteration/iterate/",
                                    stringify!($case),
                                                                            "/ours_packed_string"
                                                                        )
                                                                    )]
            fn ours_packed_string(bencher: Bencher) {
                super::iterate_packed::<$bits>(bencher, $len);
            }

            #[divan::bench(
                                                                        name = concat!(
                                    "packed_iteration/iterate/",
                                    stringify!($case),
                                                                            "/vec_u8"
                                                                        )
                                                                    )]
            fn vec_u8(bencher: Bencher) {
                super::iterate_vec::<$bits>(bencher, $len);
            }
        }
    };
}

crate::for_each_packed_bench_case!(define_case);
