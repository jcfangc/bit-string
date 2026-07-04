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

#[divan::bench(name = "count_ones/len_65/dense/ours_string")]
fn count_ones_len_65_dense_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65, Pattern::Dense);
}

#[divan::bench(name = "count_ones/len_65/dense/bitvec_simd")]
fn count_ones_len_65_dense_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 65, Pattern::Dense);
}

#[divan::bench(name = "count_ones/len_65/sparse/ours_string")]
fn count_ones_len_65_sparse_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65, Pattern::Sparse);
}

#[divan::bench(name = "count_ones/len_65/sparse/bitvec_simd")]
fn count_ones_len_65_sparse_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 65, Pattern::Sparse);
}

#[divan::bench(name = "count_ones/len_65/alternating/ours_string")]
fn count_ones_len_65_alternating_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65, Pattern::Alternating);
}

#[divan::bench(name = "count_ones/len_65/alternating/bitvec_simd")]
fn count_ones_len_65_alternating_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 65, Pattern::Alternating);
}

#[divan::bench(name = "count_ones/len_4096/dense/ours_string")]
fn count_ones_len_4096_dense_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 4096, Pattern::Dense);
}

#[divan::bench(name = "count_ones/len_4096/dense/bitvec_simd")]
fn count_ones_len_4096_dense_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 4096, Pattern::Dense);
}

#[divan::bench(name = "count_ones/len_4096/sparse/ours_string")]
fn count_ones_len_4096_sparse_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 4096, Pattern::Sparse);
}

#[divan::bench(name = "count_ones/len_4096/sparse/bitvec_simd")]
fn count_ones_len_4096_sparse_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 4096, Pattern::Sparse);
}

#[divan::bench(name = "count_ones/len_4096/alternating/ours_string")]
fn count_ones_len_4096_alternating_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 4096, Pattern::Alternating);
}

#[divan::bench(name = "count_ones/len_4096/alternating/bitvec_simd")]
fn count_ones_len_4096_alternating_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 4096, Pattern::Alternating);
}

#[divan::bench(name = "count_ones/len_65536/dense/ours_string")]
fn count_ones_len_65536_dense_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65_536, Pattern::Dense);
}

#[divan::bench(name = "count_ones/len_65536/dense/bitvec_simd")]
fn count_ones_len_65536_dense_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 65_536, Pattern::Dense);
}

#[divan::bench(name = "count_ones/len_65536/sparse/ours_string")]
fn count_ones_len_65536_sparse_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65_536, Pattern::Sparse);
}

#[divan::bench(name = "count_ones/len_65536/sparse/bitvec_simd")]
fn count_ones_len_65536_sparse_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 65_536, Pattern::Sparse);
}

#[divan::bench(name = "count_ones/len_65536/alternating/ours_string")]
fn count_ones_len_65536_alternating_bit_string(bencher: Bencher) {
    bench_bit_string(bencher, 65_536, Pattern::Alternating);
}

#[divan::bench(name = "count_ones/len_65536/alternating/bitvec_simd")]
fn count_ones_len_65536_alternating_bitvec_simd(bencher: Bencher) {
    bench_bitvec_simd(bencher, 65_536, Pattern::Alternating);
}

fn bench_bit_string(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_bit_string(len, pattern);
    bencher.bench(|| black_box(&bits).count_ones());
}

fn bench_bitvec_simd(bencher: Bencher, len: usize, pattern: Pattern) {
    let bits = make_simd_bit_vec(len, pattern);
    bencher.bench(|| black_box(&bits).count_ones());
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
