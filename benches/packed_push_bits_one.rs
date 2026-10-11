use bit_string::{PackedString, traits::PackedChar};
use divan::{Bencher, black_box};

#[derive(Clone, Copy, Eq, PartialEq)]
struct Code(u8);

impl PackedChar<1> for Code {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        (code <= 1).then_some(Self(code))
    }
}

#[derive(Clone, Copy)]
enum Pattern {
    Zero,
    One,
    Alternating,
    Random,
}

impl Pattern {
    fn codes(self, len: usize) -> Vec<u8> {
        match self {
            Self::Zero => vec![0; len],
            Self::One => vec![1; len],
            Self::Alternating => (0..len).map(|index| (index & 1) as u8).collect(),
            Self::Random => {
                let mut state = 0x9e37_79b9_u32;
                (0..len)
                    .map(|_| {
                        state ^= state << 13;
                        state ^= state >> 17;
                        state ^= state << 5;
                        (state & 1) as u8
                    })
                    .collect()
            }
        }
    }
}

fn push_batch(bencher: Bencher, len: usize, pattern: Pattern, reserve: bool) {
    let codes = pattern.codes(len);
    let reserved_seed =
        reserve.then(|| PackedString::<Code, 1>::from_chars((0..len).map(|_| Code(0))));

    bencher
        .with_inputs(|| match &reserved_seed {
            Some(seed) => {
                let mut value = seed.clone();
                value.truncate(0);
                value
            }
            None => PackedString::new(),
        })
        .bench_refs(|value| {
            let codes = black_box(codes.as_slice());
            for &code in codes {
                value.push(Code(code));
            }
            black_box(&*value);
            black_box(value.char_len())
        });
}

macro_rules! define_batch_case {
    ($module:ident, $len:literal, $pattern:ident) => {
        mod $module {
            use super::*;

            #[divan::bench(name = concat!("push_batch/", stringify!($module), "/reserved"))]
            fn reserved(bencher: Bencher) {
                crate::push_batch(bencher, $len, Pattern::$pattern, true);
            }

            #[divan::bench(name = concat!("push_batch/", stringify!($module), "/growing"))]
            fn growing(bencher: Bencher) {
                crate::push_batch(bencher, $len, Pattern::$pattern, false);
            }
        }
    };
}

fn main() {
    divan::main();
}

define_batch_case!(len_64_zero, 64, Zero);
define_batch_case!(len_64_one, 64, One);
define_batch_case!(len_64_alternating, 64, Alternating);
define_batch_case!(len_64_random, 64, Random);
define_batch_case!(len_1024_zero, 1_024, Zero);
define_batch_case!(len_1024_one, 1_024, One);
define_batch_case!(len_1024_alternating, 1_024, Alternating);
define_batch_case!(len_1024_random, 1_024, Random);
define_batch_case!(len_65536_zero, 65_536, Zero);
define_batch_case!(len_65536_one, 65_536, One);
define_batch_case!(len_65536_alternating, 65_536, Alternating);
define_batch_case!(len_65536_random, 65_536, Random);
define_batch_case!(len_1048576_zero, 1_048_576, Zero);
define_batch_case!(len_1048576_one, 1_048_576, One);
define_batch_case!(len_1048576_alternating, 1_048_576, Alternating);
define_batch_case!(len_1048576_random, 1_048_576, Random);
