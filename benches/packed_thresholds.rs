//! Compare packed construction with and without AVX2 enabled.
//!
//! Run this benchmark without `-C target-feature=+avx2` for the scalar
//! baseline, then with that target feature to measure the AVX2 dispatch.
//! Widths 2 and 8 are intentionally omitted because their AVX2 prototypes did
//! not show a stable construction-speed improvement; see the AVX2 backend
//! documentation.

#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{codes, packed};

fn main() {
    divan::main();
}

macro_rules! define_width {
    ($module:ident, $bits:literal, [$($name:ident => $len:literal),+ $(,)?]) => {
        mod $module {
            use super::*;

            $(
                #[divan::bench(
                    name = concat!(
                        "packed_thresholds/bits_",
                        stringify!($bits),
                        "/len_",
                        stringify!($len)
                    )
                )]
                fn $name(bencher: Bencher) {
                    let input = codes($bits, $len);
                    bencher.bench(|| black_box(packed::<$bits>(&input)));
                }
            )+
        }
    };
}

define_width!(
    bits_1,
    1,
    [
        len_64 => 64,
        len_128 => 128,
        len_192 => 192,
        len_256 => 256,
        len_512 => 512,
        len_1024 => 1024,
        len_4096 => 4096,
        len_16384 => 16384,
        len_1048576 => 1_048_576,
    ]
);
define_width!(
    bits_3,
    3,
    [
        len_64 => 64,
        len_128 => 128,
        len_192 => 192,
        len_256 => 256,
        len_512 => 512,
        len_1024 => 1024,
        len_4096 => 4096,
        len_16384 => 16384,
        len_1048576 => 1_048_576,
    ]
);
define_width!(
    bits_4,
    4,
    [
        len_16 => 16,
        len_32 => 32,
        len_48 => 48,
        len_64 => 64,
        len_96 => 96,
        len_128 => 128,
        len_256 => 256,
        len_512 => 512,
        len_1024 => 1024,
        len_4096 => 4096,
        len_16384 => 16384,
        len_1048576 => 1_048_576,
    ]
);
define_width!(
    bits_5,
    5,
    [
        len_64 => 64,
        len_128 => 128,
        len_192 => 192,
        len_256 => 256,
        len_512 => 512,
        len_1024 => 1024,
        len_4096 => 4096,
        len_16384 => 16384,
        len_1048576 => 1_048_576,
    ]
);
define_width!(
    bits_6,
    6,
    [
        len_16 => 16,
        len_32 => 32,
        len_48 => 48,
        len_64 => 64,
        len_96 => 96,
        len_128 => 128,
        len_256 => 256,
        len_512 => 512,
        len_1024 => 1024,
        len_4096 => 4096,
        len_16384 => 16384,
        len_1048576 => 1_048_576,
    ]
);
define_width!(
    bits_7,
    7,
    [
        len_64 => 64,
        len_128 => 128,
        len_192 => 192,
        len_256 => 256,
        len_512 => 512,
        len_1024 => 1024,
        len_4096 => 4096,
        len_16384 => 16384,
        len_1048576 => 1_048_576,
    ]
);
