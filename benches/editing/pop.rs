
// Pop from 65 bits → 64 triggers the lazy-shrink truncate_words path.
#[divan::bench(name = "pop/len_65/ours_string")]
fn pop_len_65_bit_string(bencher: Bencher) {
    pop_bench_bit_string(bencher, 65);
}

#[divan::bench(name = "pop/len_65/string")]
fn pop_len_65_string(bencher: Bencher) {
    pop_bench_string(bencher, 65);
}

#[divan::bench(name = "pop/len_65536/ours_string")]
fn pop_len_65536_bit_string(bencher: Bencher) {
    pop_bench_bit_string(bencher, 65_536);
}

#[divan::bench(name = "pop/len_65536/string")]
fn pop_len_65536_string(bencher: Bencher) {
    pop_bench_string(bencher, 65_536);
}

fn pop_bench_bit_string(bencher: Bencher, len: usize) {
    let input = pop_make_bit_string(len);

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        bits.pop();
        black_box(bits)
    });
}

fn pop_bench_string(bencher: Bencher, len: usize) {
    let input = pop_make_string(len);

    bencher.bench(|| {
        let mut s = black_box(input.clone());
        s.pop();
        black_box(s)
    });
}

#[inline]
fn pop_make_bit_string(len: usize) -> BitString {
    (0..len).map(|index| pop_bit_at(index)).collect()
}

fn pop_make_string(len: usize) -> String {
    let mut out = String::with_capacity(len);
    for index in 0..len {
        out.push(pop_bit_char(pop_bit_at(index)));
    }
    out
}

#[inline]
fn pop_bit_char(value: bool) -> char {
    if value { '1' } else { '0' }
}

#[inline]
fn pop_bit_at(index: usize) -> bool {
    pop_mix64(index as u64) & 1 != 0
}

#[inline]
fn pop_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
