use bit_string::BitString;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

// Preparation is outside timing. Repeated shifts reuse one allocation, so
// they measure steady-state throughput separately from a single operation.
fn assign(bencher: Bencher, len: usize, amount: usize, count: usize) {
    let input = make_bit_string(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|bits| {
        for _ in 0..count {
            bits.shr_assign(black_box(amount));
        }
        black_box(&*bits);
    });
}

fn owned(bencher: Bencher, len: usize, amount: usize) {
    let input = make_bit_string(len);
    // Output allocation is part of the owned operation.
    bencher.bench(|| black_box(black_box(&input).shr(black_box(amount))));
}

fn make_bit_string(len: usize) -> BitString {
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state & 1 != 0
        })
        .collect()
}

macro_rules! define_shift {
    ($case:ident, $amount:literal) => {
        mod $case {
            use super::*;

            #[divan::bench(args = [4096, 65_536, 1_048_576])]
            fn assign(bencher: Bencher, len: usize) {
                super::assign(bencher, len, $amount, 1);
            }

            #[divan::bench(args = [4096, 65_536, 1_048_576])]
            fn repeated_256(bencher: Bencher, len: usize) {
                super::assign(bencher, len, $amount, 256);
            }

            #[divan::bench(args = [4096, 65_536, 1_048_576])]
            fn owned(bencher: Bencher, len: usize) {
                super::owned(bencher, len, $amount);
            }
        }
    };
}

define_shift!(by_1, 1);
define_shift!(by_17, 17);
define_shift!(by_65, 65);
