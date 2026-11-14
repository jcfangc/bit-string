

#[derive(Clone, Copy)]
enum NotPattern {
    Dense,
    Sparse,
    Alternating,
}

#[divan::bench(name = "not/len_65/dense/ours_string")]
fn not_len_65_dense_bit_string(bencher: Bencher) {
    not_bench_bit_string(bencher, 65, NotPattern::Dense);
}

#[divan::bench(name = "not/len_65/dense/bitvec_simd")]
fn not_len_65_dense_bitvec_simd(bencher: Bencher) {
    not_bench_bitvec_simd(bencher, 65, NotPattern::Dense);
}

#[divan::bench(name = "not/len_65/sparse/ours_string")]
fn not_len_65_sparse_bit_string(bencher: Bencher) {
    not_bench_bit_string(bencher, 65, NotPattern::Sparse);
}

#[divan::bench(name = "not/len_65/sparse/bitvec_simd")]
fn not_len_65_sparse_bitvec_simd(bencher: Bencher) {
    not_bench_bitvec_simd(bencher, 65, NotPattern::Sparse);
}

#[divan::bench(name = "not/len_65/alternating/ours_string")]
fn not_len_65_alternating_bit_string(bencher: Bencher) {
    not_bench_bit_string(bencher, 65, NotPattern::Alternating);
}

#[divan::bench(name = "not/len_65/alternating/bitvec_simd")]
fn not_len_65_alternating_bitvec_simd(bencher: Bencher) {
    not_bench_bitvec_simd(bencher, 65, NotPattern::Alternating);
}

#[divan::bench(name = "not/len_4096/dense/ours_string")]
fn not_len_4096_dense_bit_string(bencher: Bencher) {
    not_bench_bit_string(bencher, 4096, NotPattern::Dense);
}

#[divan::bench(name = "not/len_4096/dense/bitvec_simd")]
fn not_len_4096_dense_bitvec_simd(bencher: Bencher) {
    not_bench_bitvec_simd(bencher, 4096, NotPattern::Dense);
}

#[divan::bench(name = "not/len_4096/sparse/ours_string")]
fn not_len_4096_sparse_bit_string(bencher: Bencher) {
    not_bench_bit_string(bencher, 4096, NotPattern::Sparse);
}

#[divan::bench(name = "not/len_4096/sparse/bitvec_simd")]
fn not_len_4096_sparse_bitvec_simd(bencher: Bencher) {
    not_bench_bitvec_simd(bencher, 4096, NotPattern::Sparse);
}

#[divan::bench(name = "not/len_4096/alternating/ours_string")]
fn not_len_4096_alternating_bit_string(bencher: Bencher) {
    not_bench_bit_string(bencher, 4096, NotPattern::Alternating);
}

#[divan::bench(name = "not/len_4096/alternating/bitvec_simd")]
fn not_len_4096_alternating_bitvec_simd(bencher: Bencher) {
    not_bench_bitvec_simd(bencher, 4096, NotPattern::Alternating);
}

#[divan::bench(name = "not/len_65536/dense/ours_string")]
fn not_len_65536_dense_bit_string(bencher: Bencher) {
    not_bench_bit_string(bencher, 65_536, NotPattern::Dense);
}

#[divan::bench(name = "not/len_65536/dense/bitvec_simd")]
fn not_len_65536_dense_bitvec_simd(bencher: Bencher) {
    not_bench_bitvec_simd(bencher, 65_536, NotPattern::Dense);
}

#[divan::bench(name = "not/len_65536/sparse/ours_string")]
fn not_len_65536_sparse_bit_string(bencher: Bencher) {
    not_bench_bit_string(bencher, 65_536, NotPattern::Sparse);
}

#[divan::bench(name = "not/len_65536/sparse/bitvec_simd")]
fn not_len_65536_sparse_bitvec_simd(bencher: Bencher) {
    not_bench_bitvec_simd(bencher, 65_536, NotPattern::Sparse);
}

#[divan::bench(name = "not/len_65536/alternating/ours_string")]
fn not_len_65536_alternating_bit_string(bencher: Bencher) {
    not_bench_bit_string(bencher, 65_536, NotPattern::Alternating);
}

#[divan::bench(name = "not/len_65536/alternating/bitvec_simd")]
fn not_len_65536_alternating_bitvec_simd(bencher: Bencher) {
    not_bench_bitvec_simd(bencher, 65_536, NotPattern::Alternating);
}

fn not_bench_bit_string(bencher: Bencher, len: usize, pattern: NotPattern) {
    let bits = not_make_bit_string(len, pattern);
    bencher.bench(|| black_box(&bits).not());
}

fn not_bench_bitvec_simd(bencher: Bencher, len: usize, pattern: NotPattern) {
    let bits = not_make_simd_bit_vec(len, pattern);
    bencher.bench(|| black_box(&bits).inverse());
}

#[inline]
fn not_make_bit_string(len: usize, pattern: NotPattern) -> BitString {
    (0..len).map(|index| not_bit_at(index, pattern)).collect()
}

#[inline]
fn not_make_simd_bit_vec(len: usize, pattern: NotPattern) -> BitVec {
    BitVec::from_bool_iterator((0..len).map(|index| not_bit_at(index, pattern)))
}

#[inline]
fn not_bit_at(index: usize, pattern: NotPattern) -> bool {
    match pattern {
        NotPattern::Dense => not_mix64(index as u64) & 1 != 0,
        NotPattern::Sparse => not_mix64(index as u64) & 63 == 0,
        NotPattern::Alternating => index % 2 != 0,
    }
}

#[inline]
fn not_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
