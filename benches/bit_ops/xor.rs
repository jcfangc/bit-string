

#[derive(Clone, Copy)]
enum XorPattern {
    Dense,
    Sparse,
    Alternating,
}

// Flattened XOR benchmarks for CodSpeed compatibility
#[divan::bench(name = "xor/len_65/dense/ours_string")]
fn xor_len_65_dense_bit_string(bencher: Bencher) {
    xor_bench_bit_string(bencher, 65, XorPattern::Dense);
}

#[divan::bench(name = "xor/len_65/dense/bitvec_simd")]
fn xor_len_65_dense_bitvec_simd(bencher: Bencher) {
    xor_bench_bitvec_simd(bencher, 65, XorPattern::Dense);
}

#[divan::bench(name = "xor/len_65/sparse/ours_string")]
fn xor_len_65_sparse_bit_string(bencher: Bencher) {
    xor_bench_bit_string(bencher, 65, XorPattern::Sparse);
}

#[divan::bench(name = "xor/len_65/sparse/bitvec_simd")]
fn xor_len_65_sparse_bitvec_simd(bencher: Bencher) {
    xor_bench_bitvec_simd(bencher, 65, XorPattern::Sparse);
}

#[divan::bench(name = "xor/len_65/alternating/ours_string")]
fn xor_len_65_alternating_bit_string(bencher: Bencher) {
    xor_bench_bit_string(bencher, 65, XorPattern::Alternating);
}

#[divan::bench(name = "xor/len_65/alternating/bitvec_simd")]
fn xor_len_65_alternating_bitvec_simd(bencher: Bencher) {
    xor_bench_bitvec_simd(bencher, 65, XorPattern::Alternating);
}

#[divan::bench(name = "xor/len_4096/dense/ours_string")]
fn xor_len_4096_dense_bit_string(bencher: Bencher) {
    xor_bench_bit_string(bencher, 4096, XorPattern::Dense);
}

#[divan::bench(name = "xor/len_4096/dense/bitvec_simd")]
fn xor_len_4096_dense_bitvec_simd(bencher: Bencher) {
    xor_bench_bitvec_simd(bencher, 4096, XorPattern::Dense);
}

#[divan::bench(name = "xor/len_4096/sparse/ours_string")]
fn xor_len_4096_sparse_bit_string(bencher: Bencher) {
    xor_bench_bit_string(bencher, 4096, XorPattern::Sparse);
}

#[divan::bench(name = "xor/len_4096/sparse/bitvec_simd")]
fn xor_len_4096_sparse_bitvec_simd(bencher: Bencher) {
    xor_bench_bitvec_simd(bencher, 4096, XorPattern::Sparse);
}

#[divan::bench(name = "xor/len_4096/alternating/ours_string")]
fn xor_len_4096_alternating_bit_string(bencher: Bencher) {
    xor_bench_bit_string(bencher, 4096, XorPattern::Alternating);
}

#[divan::bench(name = "xor/len_4096/alternating/bitvec_simd")]
fn xor_len_4096_alternating_bitvec_simd(bencher: Bencher) {
    xor_bench_bitvec_simd(bencher, 4096, XorPattern::Alternating);
}

#[divan::bench(name = "xor/len_65536/dense/ours_string")]
fn xor_len_65536_dense_bit_string(bencher: Bencher) {
    xor_bench_bit_string(bencher, 65_536, XorPattern::Dense);
}

#[divan::bench(name = "xor/len_65536/dense/bitvec_simd")]
fn xor_len_65536_dense_bitvec_simd(bencher: Bencher) {
    xor_bench_bitvec_simd(bencher, 65_536, XorPattern::Dense);
}

#[divan::bench(name = "xor/len_65536/sparse/ours_string")]
fn xor_len_65536_sparse_bit_string(bencher: Bencher) {
    xor_bench_bit_string(bencher, 65_536, XorPattern::Sparse);
}

#[divan::bench(name = "xor/len_65536/sparse/bitvec_simd")]
fn xor_len_65536_sparse_bitvec_simd(bencher: Bencher) {
    xor_bench_bitvec_simd(bencher, 65_536, XorPattern::Sparse);
}

#[divan::bench(name = "xor/len_65536/alternating/ours_string")]
fn xor_len_65536_alternating_bit_string(bencher: Bencher) {
    xor_bench_bit_string(bencher, 65_536, XorPattern::Alternating);
}

#[divan::bench(name = "xor/len_65536/alternating/bitvec_simd")]
fn xor_len_65536_alternating_bitvec_simd(bencher: Bencher) {
    xor_bench_bitvec_simd(bencher, 65_536, XorPattern::Alternating);
}

fn xor_bench_bit_string(bencher: Bencher, len: usize, pattern: XorPattern) {
    let lhs = xor_make_bit_string(len, pattern);
    let rhs = xor_make_bit_string(len, pattern);
    bencher.bench(|| black_box(&lhs).xor(black_box(&rhs)).unwrap());
}

fn xor_bench_bitvec_simd(bencher: Bencher, len: usize, pattern: XorPattern) {
    let lhs = xor_make_simd_bit_vec(len, pattern);
    let rhs = xor_make_simd_bit_vec(len, pattern);
    bencher.bench(|| black_box(&lhs).xor_cloned(black_box(&rhs)));
}

#[inline]
fn xor_make_bit_string(len: usize, pattern: XorPattern) -> BitString {
    (0..len).map(|i| xor_bit_at(i, pattern)).collect()
}

#[inline]
fn xor_make_simd_bit_vec(len: usize, pattern: XorPattern) -> BitVec {
    BitVec::from_bool_iterator((0..len).map(|i| xor_bit_at(i, pattern)))
}

#[inline]
fn xor_bit_at(index: usize, pattern: XorPattern) -> bool {
    match pattern {
        XorPattern::Dense => xor_mix64(index as u64) & 1 != 0,
        XorPattern::Sparse => xor_mix64(index as u64) & 63 == 0,
        XorPattern::Alternating => index % 2 != 0,
    }
}

#[inline]
fn xor_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
