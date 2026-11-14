
// Flattened push benchmarks for CodSpeed compatibility
#[divan::bench(name = "push/len_65/ours_string")]
fn push_len_65_bit_string(bencher: Bencher) {
    push_bench_bit_string(bencher, 65);
}

#[divan::bench(name = "push/len_65/string")]
fn push_len_65_string(bencher: Bencher) {
    push_bench_string(bencher, 65);
}

#[divan::bench(name = "push/len_65536/ours_string")]
fn push_len_65536_bit_string(bencher: Bencher) {
    push_bench_bit_string(bencher, 65_536);
}

#[divan::bench(name = "push/len_65536/string")]
fn push_len_65536_string(bencher: Bencher) {
    push_bench_string(bencher, 65_536);
}

fn push_bench_bit_string(bencher: Bencher, len: usize) {
    let input = push_make_bit_string(len);

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.push(true);
        black_box(bits)
    });
}

fn push_bench_string(bencher: Bencher, len: usize) {
    let input = push_make_string(len);

    bencher.bench(|| {
        let mut string = black_box(input.clone());
        string.push('1');
        black_box(string)
    });
}

#[inline]
fn push_make_bit_string(len: usize) -> BitString {
    (0..len).map(|index| push_bit_at(index)).collect()
}

fn push_make_string(len: usize) -> String {
    let mut out = String::with_capacity(len);
    for index in 0..len {
        out.push(push_bit_char(push_bit_at(index)));
    }
    out
}

#[inline]
fn push_bit_char(value: bool) -> char {
    if value { '1' } else { '0' }
}

#[inline]
fn push_bit_at(index: usize) -> bool {
    push_mix64(index as u64) & 1 != 0
}

#[inline]
fn push_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
