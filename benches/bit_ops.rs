use bit_string::BitString;
use bitvec_simd::BitVec;
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[derive(Clone, Copy)]
enum Pattern {
    Dense,
    Sparse,
    Alternating,
}

macro_rules! bitvec_pair_case {
    ($name:ident, $len:expr, $pattern:expr, $bit_fn:ident, $simd_fn:ident) => {
        #[divan::bench_group]
        mod $name {
            use super::*;

            #[divan::bench]
            fn bit_string(bencher: Bencher) {
                $bit_fn(bencher, $len, $pattern);
            }

            #[divan::bench]
            fn bitvec_simd(bencher: Bencher) {
                $simd_fn(bencher, $len, $pattern);
            }
        }
    };
}

macro_rules! bitvec_pattern_cases {
    ($bit_fn:ident, $simd_fn:ident) => {
        bitvec_pair_case!(len_65_dense, 65, Pattern::Dense, $bit_fn, $simd_fn);
        bitvec_pair_case!(len_65_sparse, 65, Pattern::Sparse, $bit_fn, $simd_fn);
        bitvec_pair_case!(
            len_65_alternating,
            65,
            Pattern::Alternating,
            $bit_fn,
            $simd_fn
        );

        bitvec_pair_case!(len_4096_dense, 4096, Pattern::Dense, $bit_fn, $simd_fn);
        bitvec_pair_case!(len_4096_sparse, 4096, Pattern::Sparse, $bit_fn, $simd_fn);
        bitvec_pair_case!(
            len_4096_alternating,
            4096,
            Pattern::Alternating,
            $bit_fn,
            $simd_fn
        );

        bitvec_pair_case!(len_65536_dense, 65_536, Pattern::Dense, $bit_fn, $simd_fn);
        bitvec_pair_case!(len_65536_sparse, 65_536, Pattern::Sparse, $bit_fn, $simd_fn);
        bitvec_pair_case!(
            len_65536_alternating,
            65_536,
            Pattern::Alternating,
            $bit_fn,
            $simd_fn
        );
    };
}

macro_rules! bit_string_shift_case {
    ($name:ident, $len:expr, $amount:expr, $bench_fn:ident) => {
        #[divan::bench_group]
        mod $name {
            use super::*;

            #[divan::bench]
            fn bit_string(bencher: Bencher) {
                $bench_fn(bencher, $len, $amount);
            }
        }
    };
}

macro_rules! shift_cases {
    ($bench_fn:ident) => {
        bit_string_shift_case!(len_65_amount_1, 65, 1, $bench_fn);

        bit_string_shift_case!(len_4096_amount_1, 4096, 1, $bench_fn);
        bit_string_shift_case!(len_4096_amount_65, 4096, 65, $bench_fn);

        bit_string_shift_case!(len_65536_amount_1, 65_536, 1, $bench_fn);
        bit_string_shift_case!(len_65536_amount_65, 65_536, 65, $bench_fn);
    };
}

#[divan::bench_group]
mod and {
    use super::*;
    bitvec_pattern_cases!(bench_bit_string_and_bits, bench_bitvec_simd_and_cloned);
}

#[divan::bench_group]
mod or {
    use super::*;
    bitvec_pattern_cases!(bench_bit_string_or_bits, bench_bitvec_simd_or_cloned);
}

#[divan::bench_group]
mod xor {
    use super::*;
    bitvec_pattern_cases!(bench_bit_string_xor_bits, bench_bitvec_simd_xor_cloned);
}

#[divan::bench_group]
mod not {
    use super::*;
    bitvec_pattern_cases!(bench_bit_string_not_bits, bench_bitvec_simd_inverse);
}

#[divan::bench_group]
mod count_ones {
    use super::*;
    bitvec_pattern_cases!(bench_bit_string_count_ones, bench_bitvec_simd_count_ones);
}

#[divan::bench_group]
mod count_zeros {
    use super::*;
    bitvec_pattern_cases!(
        bench_bit_string_count_zeros,
        bench_bitvec_simd_count_zeros_derived
    );
}

#[divan::bench_group]
mod shl {
    use super::*;
    shift_cases!(bench_bit_string_shl_zeros);
}

#[divan::bench_group]
mod shr {
    use super::*;
    shift_cases!(bench_bit_string_shr_zeros);
}

fn bench_bit_string_and_bits(bencher: Bencher, len: usize, pattern: Pattern) {
    let lhs = make_bit_string(len, pattern);
    let rhs = make_bit_string(len, pattern);

    bencher.bench(|| black_box(&lhs).and_bits(black_box(&rhs)).unwrap());
}

fn bench_bitvec_simd_and_cloned(bencher: Bencher, len: usize, pattern: Pattern) {
    let lhs = make_simd_bit_vec(len, pattern);
    let rhs = make_simd_bit_vec(len, pattern);

    bencher.bench(|| black_box(&lhs).and_cloned(black_box(&rhs)));
}

fn bench_bit_string_or_bits(bencher: Bencher, len: usize, pattern: Pattern) {
    let lhs = make_bit_string(len, pattern);
    let rhs = make_bit_string(len, pattern);

    bencher.bench(|| black_box(&lhs).or_bits(black_box(&rhs)).unwrap());
}

fn bench_bitvec_simd_or_cloned(bencher: Bencher, len: usize, pattern: Pattern) {
    let lhs = make_simd_bit_vec(len, pattern);
    let rhs = make_simd_bit_vec(len, pattern);

    bencher.bench(|| black_box(&lhs).or_cloned(black_box(&rhs)));
}

fn bench_bit_string_xor_bits(bencher: Bencher, len: usize, pattern: Pattern) {
    let lhs = make_bit_string(len, pattern);
    let rhs = make_bit_string(len, pattern);

    bencher.bench(|| black_box(&lhs).xor_bits(black_box(&rhs)).unwrap());
}

fn bench_bitvec_simd_xor_cloned(bencher: Bencher, len: usize, pattern: Pattern) {
    let lhs = make_simd_bit_vec(len, pattern);
    let rhs = make_simd_bit_vec(len, pattern);

    bencher.bench(|| black_box(&lhs).xor_cloned(black_box(&rhs)));
}

fn bench_bit_string_not_bits(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);

    bencher.bench(|| black_box(&bits).not_bits());
}

fn bench_bitvec_simd_inverse(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_simd_bit_vec(len, pattern);

    bencher.bench(|| black_box(&bits).inverse());
}

fn bench_bit_string_count_ones(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);

    bencher.bench(|| black_box(&bits).count_ones());
}

fn bench_bitvec_simd_count_ones(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_simd_bit_vec(len, pattern);

    bencher.bench(|| black_box(&bits).count_ones());
}

fn bench_bit_string_count_zeros(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);

    bencher.bench(|| black_box(&bits).count_zeros());
}

fn bench_bitvec_simd_count_zeros_derived(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_simd_bit_vec(len, pattern);

    bencher.bench(|| {
        let bits = black_box(&bits);
        bits.len() - bits.count_ones()
    });
}

fn bench_bit_string_shl_zeros(bencher: Bencher, len: usize, amount: usize) {
    let bits = make_bit_string(len, Pattern::Dense);

    bencher.bench(|| black_box(&bits).shl_zeros(black_box(amount)));
}

fn bench_bit_string_shr_zeros(bencher: Bencher, len: usize, amount: usize) {
    let bits = make_bit_string(len, Pattern::Dense);

    bencher.bench(|| black_box(&bits).shr_zeros(black_box(amount)));
}

#[inline]
fn make_bit_string(len: usize, pattern: Pattern) -> BitString {
    (0..len).map(|index| bit_at(index, pattern)).collect()
}

#[inline]
fn make_simd_bit_vec(len: usize, pattern: Pattern) -> BitVec {
    BitVec::from_bool_iterator((0..len).map(|index| bit_at(index, pattern)))
}

#[inline]
fn bit_at(index: usize, pattern: Pattern) -> bool {
    match pattern {
        Pattern::Dense => mix64(index as u64) & 1 != 0,
        Pattern::Sparse => mix64(index as u64) & 63 == 0,
        Pattern::Alternating => index % 2 != 0,
    }
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
