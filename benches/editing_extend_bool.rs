use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

struct UnknownSize<I>(I);

impl<I: Iterator> Iterator for UnknownSize<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

fn input_bools(len: usize) -> Vec<bool> {
    (0..len).map(|index| index % 3 != 0).collect()
}

fn prefix(len: usize, reserve_tail: usize) -> BitString {
    let mut bits = BitString::zeros(len.saturating_add(reserve_tail));
    bits.truncate(len);
    bits
}

fn exact(bencher: Bencher, prefix_len: usize, append_len: usize, reserve_tail: usize) {
    let values = input_bools(append_len);
    bencher
        .with_inputs(|| prefix(prefix_len, reserve_tail))
        .bench_refs(|bits| {
            bits.extend(black_box(values.as_slice()).iter().copied());
            black_box(&*bits);
        });
}

fn unknown(bencher: Bencher, prefix_len: usize, append_len: usize, reserve_tail: usize) {
    let values = input_bools(append_len);
    bencher
        .with_inputs(|| prefix(prefix_len, reserve_tail))
        .bench_refs(|bits| {
            let iter = black_box(values.as_slice()).iter().copied();
            bits.extend(UnknownSize(iter));
            black_box(&*bits);
        });
}

macro_rules! define_case {
    ($module:ident, $prefix:literal, $append:literal) => {
        mod $module {
            use super::*;

            #[divan::bench(name = concat!("extend_bool/prefix_", stringify!($prefix), "/append_", stringify!($append), "/exact_size_hint/ours_string"))]
            fn exact_size_hint(bencher: Bencher) {
                super::exact(bencher, $prefix, $append, 0);
            }

            #[divan::bench(name = concat!("extend_bool/prefix_", stringify!($prefix), "/append_", stringify!($append), "/unknown_size_hint/ours_string"))]
            fn unknown_size_hint(bencher: Bencher) {
                super::unknown(bencher, $prefix, $append, 0);
            }
        }
    };
}

macro_rules! define_reserved_case {
    ($module:ident, $prefix:literal, $append:literal) => {
        mod $module {
            use super::*;

            #[divan::bench(name = concat!("extend_bool/prefix_", stringify!($prefix), "/append_", stringify!($append), "/reserved/exact_size_hint/ours_string"))]
            fn exact_size_hint(bencher: Bencher) {
                super::exact(bencher, $prefix, $append, $append);
            }

            #[divan::bench(name = concat!("extend_bool/prefix_", stringify!($prefix), "/append_", stringify!($append), "/reserved/unknown_size_hint/ours_string"))]
            fn unknown_size_hint(bencher: Bencher) {
                super::unknown(bencher, $prefix, $append, $append);
            }
        }
    };
}

#[cfg(feature = "characterization")]
mod full_matrix {
    use super::*;

    define_case!(prefix_0_append_0, 0, 0);
    define_case!(prefix_0_append_1, 0, 1);
    define_case!(prefix_0_append_63, 0, 63);
    define_case!(prefix_0_append_64, 0, 64);
    define_case!(prefix_0_append_65, 0, 65);
    define_case!(prefix_0_append_4096, 0, 4096);
    define_case!(prefix_0_append_65536, 0, 65536);
    define_case!(prefix_63_append_0, 63, 0);
    define_case!(prefix_63_append_1, 63, 1);
    define_case!(prefix_63_append_63, 63, 63);
    define_case!(prefix_63_append_64, 63, 64);
    define_case!(prefix_63_append_65, 63, 65);
    define_case!(prefix_63_append_4096, 63, 4096);
    define_case!(prefix_63_append_65536, 63, 65536);
    define_case!(prefix_64_append_0, 64, 0);
    define_case!(prefix_64_append_1, 64, 1);
    define_case!(prefix_64_append_63, 64, 63);
    define_case!(prefix_64_append_64, 64, 64);
    define_case!(prefix_64_append_65, 64, 65);
    define_case!(prefix_64_append_4096, 64, 4096);
    define_case!(prefix_64_append_65536, 64, 65536);
    define_case!(prefix_65_append_0, 65, 0);
    define_case!(prefix_65_append_1, 65, 1);
    define_case!(prefix_65_append_63, 65, 63);
    define_case!(prefix_65_append_64, 65, 64);
    define_case!(prefix_65_append_65, 65, 65);
    define_case!(prefix_65_append_4096, 65, 4096);
    define_case!(prefix_65_append_65536, 65, 65536);
    define_case!(prefix_4096_append_0, 4096, 0);
    define_case!(prefix_4096_append_1, 4096, 1);
    define_case!(prefix_4096_append_63, 4096, 63);
    define_case!(prefix_4096_append_64, 4096, 64);
    define_case!(prefix_4096_append_65, 4096, 65);
    define_case!(prefix_4096_append_4096, 4096, 4096);
    define_case!(prefix_4096_append_65536, 4096, 65536);

    define_reserved_case!(reserved_4096_1, 4096, 1);
    define_reserved_case!(reserved_4096_63, 4096, 63);
    define_reserved_case!(reserved_4096_64, 4096, 64);
    define_reserved_case!(reserved_4096_65, 4096, 65);
    define_reserved_case!(reserved_4096_4096, 4096, 4096);
    define_reserved_case!(reserved_65536_1, 65536, 1);
    define_reserved_case!(reserved_65536_63, 65536, 63);
    define_reserved_case!(reserved_65536_64, 65536, 64);
    define_reserved_case!(reserved_65536_65, 65536, 65);
    define_reserved_case!(reserved_65536_4096, 65536, 4096);
    define_reserved_case!(reserved_65536_65536, 65536, 65536);
}

#[cfg(not(feature = "characterization"))]
define_case!(prefix_0_append_0, 0, 0);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_0_append_1, 0, 1);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_0_append_63, 0, 63);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_0_append_64, 0, 64);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_0_append_65, 0, 65);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_0_append_4096, 0, 4096);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_0_append_65536, 0, 65536);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_65_append_0, 65, 0);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_65_append_1, 65, 1);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_65_append_63, 65, 63);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_65_append_64, 65, 64);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_65_append_65, 65, 65);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_65_append_4096, 65, 4096);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_65_append_65536, 65, 65536);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_4096_append_0, 4096, 0);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_4096_append_1, 4096, 1);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_4096_append_63, 4096, 63);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_4096_append_64, 4096, 64);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_4096_append_65, 4096, 65);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_4096_append_4096, 4096, 4096);
#[cfg(not(feature = "characterization"))]
define_case!(prefix_4096_append_65536, 4096, 65536);

#[cfg(not(feature = "characterization"))]
define_reserved_case!(reserved_4096_64, 4096, 64);
#[cfg(not(feature = "characterization"))]
define_reserved_case!(reserved_4096_4096, 4096, 4096);
#[cfg(not(feature = "characterization"))]
define_reserved_case!(reserved_65536_64, 65536, 64);
#[cfg(not(feature = "characterization"))]
define_reserved_case!(reserved_65536_65536, 65536, 65536);
