
#[derive(Clone, Copy)]
enum CountOnesPattern {
    Dense,
    Sparse,
    Alternating,
}

#[divan::bench(name = "count_ones/len_65/dense/ours_string")]
fn count_ones_len_65_dense_bit_string(bencher: Bencher) {
    count_ones_bench_bit_string(bencher, 65, CountOnesPattern::Dense);
}

#[divan::bench(name = "count_ones/len_65/dense/bitvec_simd")]
fn count_ones_len_65_dense_bitvec_simd(bencher: Bencher) {
    count_ones_bench_bitvec_simd(bencher, 65, CountOnesPattern::Dense);
}

#[divan::bench(name = "count_ones/len_65/sparse/ours_string")]
fn count_ones_len_65_sparse_bit_string(bencher: Bencher) {
    count_ones_bench_bit_string(bencher, 65, CountOnesPattern::Sparse);
}

#[divan::bench(name = "count_ones/len_65/sparse/bitvec_simd")]
fn count_ones_len_65_sparse_bitvec_simd(bencher: Bencher) {
    count_ones_bench_bitvec_simd(bencher, 65, CountOnesPattern::Sparse);
}

#[divan::bench(name = "count_ones/len_65/alternating/ours_string")]
fn count_ones_len_65_alternating_bit_string(bencher: Bencher) {
    count_ones_bench_bit_string(bencher, 65, CountOnesPattern::Alternating);
}

#[divan::bench(name = "count_ones/len_65/alternating/bitvec_simd")]
fn count_ones_len_65_alternating_bitvec_simd(bencher: Bencher) {
    count_ones_bench_bitvec_simd(bencher, 65, CountOnesPattern::Alternating);
}

#[divan::bench(name = "count_ones/len_4096/dense/ours_string")]
fn count_ones_len_4096_dense_bit_string(bencher: Bencher) {
    count_ones_bench_bit_string(bencher, 4096, CountOnesPattern::Dense);
}

#[divan::bench(name = "count_ones/len_4096/dense/bitvec_simd")]
fn count_ones_len_4096_dense_bitvec_simd(bencher: Bencher) {
    count_ones_bench_bitvec_simd(bencher, 4096, CountOnesPattern::Dense);
}

#[divan::bench(name = "count_ones/len_4096/sparse/ours_string")]
fn count_ones_len_4096_sparse_bit_string(bencher: Bencher) {
    count_ones_bench_bit_string(bencher, 4096, CountOnesPattern::Sparse);
}

#[divan::bench(name = "count_ones/len_4096/sparse/bitvec_simd")]
fn count_ones_len_4096_sparse_bitvec_simd(bencher: Bencher) {
    count_ones_bench_bitvec_simd(bencher, 4096, CountOnesPattern::Sparse);
}

#[divan::bench(name = "count_ones/len_4096/alternating/ours_string")]
fn count_ones_len_4096_alternating_bit_string(bencher: Bencher) {
    count_ones_bench_bit_string(bencher, 4096, CountOnesPattern::Alternating);
}

#[divan::bench(name = "count_ones/len_4096/alternating/bitvec_simd")]
fn count_ones_len_4096_alternating_bitvec_simd(bencher: Bencher) {
    count_ones_bench_bitvec_simd(bencher, 4096, CountOnesPattern::Alternating);
}

#[divan::bench(name = "count_ones/len_65536/dense/ours_string")]
fn count_ones_len_65536_dense_bit_string(bencher: Bencher) {
    count_ones_bench_bit_string(bencher, 65_536, CountOnesPattern::Dense);
}

#[divan::bench(name = "count_ones/len_65536/dense/bitvec_simd")]
fn count_ones_len_65536_dense_bitvec_simd(bencher: Bencher) {
    count_ones_bench_bitvec_simd(bencher, 65_536, CountOnesPattern::Dense);
}

#[divan::bench(name = "count_ones/len_65536/sparse/ours_string")]
fn count_ones_len_65536_sparse_bit_string(bencher: Bencher) {
    count_ones_bench_bit_string(bencher, 65_536, CountOnesPattern::Sparse);
}

#[divan::bench(name = "count_ones/len_65536/sparse/bitvec_simd")]
fn count_ones_len_65536_sparse_bitvec_simd(bencher: Bencher) {
    count_ones_bench_bitvec_simd(bencher, 65_536, CountOnesPattern::Sparse);
}

#[divan::bench(name = "count_ones/len_65536/alternating/ours_string")]
fn count_ones_len_65536_alternating_bit_string(bencher: Bencher) {
    count_ones_bench_bit_string(bencher, 65_536, CountOnesPattern::Alternating);
}

#[divan::bench(name = "count_ones/len_65536/alternating/bitvec_simd")]
fn count_ones_len_65536_alternating_bitvec_simd(bencher: Bencher) {
    count_ones_bench_bitvec_simd(bencher, 65_536, CountOnesPattern::Alternating);
}

fn count_ones_bench_bit_string(bencher: Bencher, len: usize, pattern: CountOnesPattern) {
    let bits = count_ones_make_bit_string(len, pattern);
    bencher.bench(|| black_box(&bits).count_ones());
}

fn count_ones_bench_bitvec_simd(bencher: Bencher, len: usize, pattern: CountOnesPattern) {
    let bits = count_ones_make_simd_bit_vec(len, pattern);
    bencher.bench(|| black_box(&bits).count_ones());
}

#[inline]
fn count_ones_make_bit_string(len: usize, pattern: CountOnesPattern) -> BitString {
    (0..len).map(|index| count_ones_bit_at(index, pattern)).collect()
}

#[inline]
fn count_ones_make_simd_bit_vec(len: usize, pattern: CountOnesPattern) -> BitVec {
    BitVec::from_bool_iterator((0..len).map(|index| count_ones_bit_at(index, pattern)))
}

#[inline]
fn count_ones_bit_at(index: usize, pattern: CountOnesPattern) -> bool {
    match pattern {
        CountOnesPattern::Dense => count_ones_mix64(index as u64) & 1 != 0,
        CountOnesPattern::Sparse => count_ones_mix64(index as u64) & 63 == 0,
        CountOnesPattern::Alternating => index % 2 != 0,
    }
}

#[inline]
fn count_ones_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
