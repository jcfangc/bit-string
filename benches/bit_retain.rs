use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum RetainMode {
    KeepAll,
    KeepOnes,
    KeepZeros,
    KeepNone,
}

impl RetainMode {
    fn keeps(self, value: bool) -> bool {
        match self {
            Self::KeepAll => true,
            Self::KeepOnes => value,
            Self::KeepZeros => !value,
            Self::KeepNone => false,
        }
    }
}

fn input(len: usize) -> BitString {
    let words = (0..len.div_ceil(64))
        .map(|index| {
            let mut value = index as u64 + 0x9e37_79b9_7f4a_7c15;
            value ^= value >> 12;
            value ^= value << 25;
            value ^= value >> 27;
            value.wrapping_mul(0x2545_f491_4f6c_dd1d)
        })
        .collect::<Vec<_>>();
    BitString::from_words(&words, len).expect("word count matches bit length")
}

fn retain_production(bencher: Bencher, len: usize, mode: RetainMode) {
    let input = input(len);
    assert_retain_matches_reference(&input, mode);

    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        black_box(&mut *value).retain(|bit| mode.keeps(black_box(bit)));
        black_box(&*value);
        black_box(value.bit_len());
    });
}

fn assert_retain_matches_reference(input: &BitString, mode: RetainMode) {
    let expected = input
        .iter()
        .filter(|&bit| mode.keeps(bit))
        .collect::<Vec<_>>();
    let mut actual = input.clone();
    actual.retain(|bit| mode.keeps(bit));
    assert_eq!(actual.to_bool_vec(), expected);
}

macro_rules! define_case {
    ($case:ident, $len:literal) => {
        mod $case {
            use super::*;

            macro_rules! define_mode {
                ($name:ident, $mode:expr) => {
                    #[divan::bench(name = concat!("bit_retain/", stringify!($case), "/", stringify!($name), "/ours_bit"))]
                    fn $name(bencher: Bencher) {
                        retain_production(bencher, $len, $mode);
                    }
                };
            }

            define_mode!(keep_all, RetainMode::KeepAll);
            define_mode!(keep_ones, RetainMode::KeepOnes);
            define_mode!(keep_zeros, RetainMode::KeepZeros);
            define_mode!(keep_none, RetainMode::KeepNone);
        }
    };
}

define_case!(len_63, 63);
define_case!(len_64, 64);
define_case!(len_65, 65);
define_case!(len_4096, 4096);
define_case!(len_65536, 65536);
