#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{codes, packed};

fn main() {
    divan::main();
}

fn construct_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| black_box(packed::<BITS>(&input)));
}

fn construct_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.bench(|| black_box(input.iter().copied().collect::<Vec<_>>()));
}

macro_rules! define_case {
    ($case:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(
                                                        name = concat!(
                                                            "packed_construction/from_chars/",
                                                            stringify!($case),
                                                            "/ours_packed_string"
                                                        )
                                                    )]
            fn ours_packed_string(bencher: Bencher) {
                super::construct_packed::<$bits>(bencher, $len);
            }

            #[divan::bench(
                                                        name = concat!(
                                                            "packed_construction/from_chars/",
                                                            stringify!($case),
                                                            "/vec_u8"
                                                        )
                                                    )]
            fn vec_u8(bencher: Bencher) {
                super::construct_vec::<$bits>(bencher, $len);
            }
        }
    };
}

crate::for_each_packed_bench_case!(define_case);
