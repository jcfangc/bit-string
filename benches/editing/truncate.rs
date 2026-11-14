
// Truncate 65→64 crosses a word boundary, testing the lazy-shrink fast path.
#[divan::bench(name = "truncate/65_to_64/ours_string")]
fn truncate_65_to_64_bit_string(bencher: Bencher) {
    truncate_bench_bit_string(bencher, 65, 64);
}

#[divan::bench(name = "truncate/65_to_64/string")]
fn truncate_65_to_64_string(bencher: Bencher) {
    truncate_bench_string(bencher, 65, 64);
}

// Truncate 65536→128 removes many words, tests batch-shrink behavior.
#[divan::bench(name = "truncate/65536_to_128/ours_string")]
fn truncate_65536_to_128_bit_string(bencher: Bencher) {
    truncate_bench_bit_string(bencher, 65_536, 128);
}

#[divan::bench(name = "truncate/65536_to_128/string")]
fn truncate_65536_to_128_string(bencher: Bencher) {
    truncate_bench_string(bencher, 65_536, 128);
}

// Truncate 128→65 partial-word rewrite in the last word.
#[divan::bench(name = "truncate/128_to_65/ours_string")]
fn truncate_128_to_65_bit_string(bencher: Bencher) {
    truncate_bench_bit_string(bencher, 128, 65);
}

#[divan::bench(name = "truncate/128_to_65/string")]
fn truncate_128_to_65_string(bencher: Bencher) {
    truncate_bench_string(bencher, 128, 65);
}

fn truncate_bench_bit_string(bencher: Bencher, len: usize, target: usize) {
    let input = truncate_make_bit_string(len);

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.truncate(target);
        black_box(bits)
    });
}

fn truncate_bench_string(bencher: Bencher, len: usize, target: usize) {
    let input = truncate_make_string(len);

    bencher.bench(|| {
        let mut s = black_box(input.clone());
        s.truncate(target);
        black_box(s)
    });
}

#[inline]
fn truncate_make_bit_string(len: usize) -> BitString {
    (0..len).map(|index| truncate_bit_at(index)).collect()
}

fn truncate_make_string(len: usize) -> String {
    let mut out = String::with_capacity(len);
    for index in 0..len {
        out.push(truncate_bit_char(truncate_bit_at(index)));
    }
    out
}

#[inline]
fn truncate_bit_char(value: bool) -> char {
    if value { '1' } else { '0' }
}

#[inline]
fn truncate_bit_at(index: usize) -> bool {
    truncate_mix64(index as u64) & 1 != 0
}

#[inline]
fn truncate_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
