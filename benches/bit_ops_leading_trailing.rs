use bit_string::BitString;
use bitvec_simd::BitVec;
use divan::{Bencher, black_box};
use int_interval::UsizeCO;

fn main() {
    divan::main();
}

// ===========================================================================
// Patterns
// ===========================================================================

#[derive(Clone, Copy)]
enum Pattern {
    AllZeros,
    Alternating,
    Dense,
}

// ===========================================================================
// leading_zeros
// ===========================================================================

mod leading_zeros {
    use super::*;

    #[divan::bench(name = "leading_zeros/len_65/all_zeros/ours_string")]
    fn len_65_all_zeros_bit_string(b: Bencher) {
        bench_leading_zeros(b, 65, Pattern::AllZeros);
    }

    #[divan::bench(name = "leading_zeros/len_65/all_zeros/bitvec_simd")]
    fn len_65_all_zeros_bitvec_simd(b: Bencher) {
        bench_leading_zeros_bitvec(b, 65, Pattern::AllZeros);
    }

    #[divan::bench(name = "leading_zeros/len_65/alternating/ours_string")]
    fn len_65_alternating_bit_string(b: Bencher) {
        bench_leading_zeros(b, 65, Pattern::Alternating);
    }

    #[divan::bench(name = "leading_zeros/len_65/alternating/bitvec_simd")]
    fn len_65_alternating_bitvec_simd(b: Bencher) {
        bench_leading_zeros_bitvec(b, 65, Pattern::Alternating);
    }

    #[divan::bench(name = "leading_zeros/len_65/dense/ours_string")]
    fn len_65_dense_bit_string(b: Bencher) {
        bench_leading_zeros(b, 65, Pattern::Dense);
    }

    #[divan::bench(name = "leading_zeros/len_65/dense/bitvec_simd")]
    fn len_65_dense_bitvec_simd(b: Bencher) {
        bench_leading_zeros_bitvec(b, 65, Pattern::Dense);
    }

    #[divan::bench(name = "leading_zeros/len_4096/all_zeros/ours_string")]
    fn len_4096_all_zeros_bit_string(b: Bencher) {
        bench_leading_zeros(b, 4096, Pattern::AllZeros);
    }

    #[divan::bench(name = "leading_zeros/len_4096/all_zeros/bitvec_simd")]
    fn len_4096_all_zeros_bitvec_simd(b: Bencher) {
        bench_leading_zeros_bitvec(b, 4096, Pattern::AllZeros);
    }

    #[divan::bench(name = "leading_zeros/len_4096/dense/ours_string")]
    fn len_4096_dense_bit_string(b: Bencher) {
        bench_leading_zeros(b, 4096, Pattern::Dense);
    }

    #[divan::bench(name = "leading_zeros/len_4096/dense/bitvec_simd")]
    fn len_4096_dense_bitvec_simd(b: Bencher) {
        bench_leading_zeros_bitvec(b, 4096, Pattern::Dense);
    }

    #[divan::bench(name = "leading_zeros/len_65536/all_zeros/ours_string")]
    fn len_65536_all_zeros_bit_string(b: Bencher) {
        bench_leading_zeros(b, 65536, Pattern::AllZeros);
    }

    #[divan::bench(name = "leading_zeros/len_65536/all_zeros/bitvec_simd")]
    fn len_65536_all_zeros_bitvec_simd(b: Bencher) {
        bench_leading_zeros_bitvec(b, 65536, Pattern::AllZeros);
    }
}

// ===========================================================================
// leading_zeros — unaligned BitStr views (no bitvec_simd equivalent)
// ===========================================================================

mod leading_zeros_unaligned {
    use super::*;

    #[divan::bench(name = "leading_zeros/unaligned_3/len_4096/all_zeros")]
    fn unaligned_3_len_4096_all_zeros(b: Bencher) {
        bench_leading_zeros_unaligned(b, 4096, 3, Pattern::AllZeros);
    }

    #[divan::bench(name = "leading_zeros/unaligned_31/len_4096/all_zeros")]
    fn unaligned_31_len_4096_all_zeros(b: Bencher) {
        bench_leading_zeros_unaligned(b, 4096, 31, Pattern::AllZeros);
    }

    #[divan::bench(name = "leading_zeros/unaligned_63/len_4096/all_zeros")]
    fn unaligned_63_len_4096_all_zeros(b: Bencher) {
        bench_leading_zeros_unaligned(b, 4096, 63, Pattern::AllZeros);
    }
}

// ===========================================================================
// trailing_zeros (no bitvec_simd equivalent)
// ===========================================================================

mod trailing_zeros {
    use super::*;

    #[divan::bench(name = "trailing_zeros/len_65/all_zeros")]
    fn len_65_all_zeros(b: Bencher) {
        bench_trailing_zeros(b, 65, Pattern::AllZeros);
    }

    #[divan::bench(name = "trailing_zeros/len_65/alternating")]
    fn len_65_alternating(b: Bencher) {
        bench_trailing_zeros(b, 65, Pattern::Alternating);
    }

    #[divan::bench(name = "trailing_zeros/len_65/dense")]
    fn len_65_dense(b: Bencher) {
        bench_trailing_zeros(b, 65, Pattern::Dense);
    }

    #[divan::bench(name = "trailing_zeros/len_4096/all_zeros")]
    fn len_4096_all_zeros(b: Bencher) {
        bench_trailing_zeros(b, 4096, Pattern::AllZeros);
    }

    #[divan::bench(name = "trailing_zeros/len_4096/dense")]
    fn len_4096_dense(b: Bencher) {
        bench_trailing_zeros(b, 4096, Pattern::Dense);
    }

    #[divan::bench(name = "trailing_zeros/len_65536/all_zeros")]
    fn len_65536_all_zeros(b: Bencher) {
        bench_trailing_zeros(b, 65536, Pattern::AllZeros);
    }

    #[divan::bench(name = "trailing_zeros/len_65536/dense")]
    fn len_65536_dense(b: Bencher) {
        bench_trailing_zeros(b, 65536, Pattern::Dense);
    }
}

// ===========================================================================
// trailing_zeros — unaligned BitStr views
// ===========================================================================

mod trailing_zeros_unaligned {
    use super::*;

    #[divan::bench(name = "trailing_zeros/unaligned_3/len_4096/all_zeros")]
    fn unaligned_3_len_4096_all_zeros(b: Bencher) {
        bench_trailing_zeros_unaligned(b, 4096, 3, Pattern::AllZeros);
    }

    #[divan::bench(name = "trailing_zeros/unaligned_63/len_4096/all_zeros")]
    fn unaligned_63_len_4096_all_zeros(b: Bencher) {
        bench_trailing_zeros_unaligned(b, 4096, 63, Pattern::AllZeros);
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

fn make_bit_string(len: usize, pattern: Pattern) -> BitString {
    (0..len).map(|index| bit_at(index, pattern)).collect()
}

fn make_bitvec_simd(len: usize, pattern: Pattern) -> BitVec {
    BitVec::from_bool_iterator((0..len).map(|index| bit_at(index, pattern)))
}

fn bench_leading_zeros(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);
    let view = bits.as_bit_str();
    bencher.bench(|| black_box(&view).leading_zeros());
}

fn bench_leading_zeros_bitvec(bencher: Bencher, len: usize, pattern: Pattern) {
    let bv = make_bitvec_simd(len, pattern);
    bencher.bench(|| black_box(&bv).leading_zeros());
}

fn bench_leading_zeros_unaligned(bencher: Bencher, len: usize, skip: usize, pattern: Pattern) {
    let pad = skip + len + 10;
    let mut bits = BitString::zeros(pad);
    for i in skip..skip + len {
        bits.set(i, bit_at(i - skip, pattern));
    }
    let view = bits.as_bit_str();
    let sub = view.slice(UsizeCO::try_new(skip, skip + len).unwrap());
    bencher.bench(|| black_box(&sub).leading_zeros());
}

fn bench_trailing_zeros(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);
    let view = bits.as_bit_str();
    bencher.bench(|| black_box(&view).trailing_zeros());
}

fn bench_trailing_zeros_unaligned(bencher: Bencher, len: usize, skip: usize, pattern: Pattern) {
    let pad = skip + len + 10;
    let mut bits = BitString::zeros(pad);
    for i in skip..skip + len {
        bits.set(i, bit_at(i - skip, pattern));
    }
    let view = bits.as_bit_str();
    let sub = view.slice(UsizeCO::try_new(skip, skip + len).unwrap());
    bencher.bench(|| black_box(&sub).trailing_zeros());
}

#[inline]
fn bit_at(index: usize, pattern: Pattern) -> bool {
    match pattern {
        Pattern::AllZeros => false,
        Pattern::Alternating => index % 2 != 0,
        Pattern::Dense => mix64(index as u64) & 1 != 0,
    }
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
