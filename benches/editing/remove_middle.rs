
#[divan::bench(name = "remove_middle/len_65/ours_string")]
fn remove_middle_len_65_bit_string(bencher: Bencher) {
    remove_middle_bench_bit_string(bencher, 65);
}

#[divan::bench(name = "remove_middle/len_65/string")]
fn remove_middle_len_65_string(bencher: Bencher) {
    remove_middle_bench_string(bencher, 65);
}

#[divan::bench(name = "remove_middle/len_65536/ours_string")]
fn remove_middle_len_65536_bit_string(bencher: Bencher) {
    remove_middle_bench_bit_string(bencher, 65_536);
}

#[divan::bench(name = "remove_middle/len_65536/string")]
fn remove_middle_len_65536_string(bencher: Bencher) {
    remove_middle_bench_string(bencher, 65_536);
}

fn remove_middle_bench_bit_string(bencher: Bencher, len: usize) {
    let input = remove_middle_make_bit_string(len);
    let index = len / 2;

    bencher.bench(|| {
        let mut bits = black_box(input.clone());
        black_box(bits.remove(black_box(index)));
        black_box(bits)
    });
}

fn remove_middle_bench_string(bencher: Bencher, len: usize) {
    let input = remove_middle_make_string(len);
    let index = len / 2;

    bencher.bench(|| {
        let mut string = black_box(input.clone());
        black_box(string.remove(black_box(index)));
        black_box(string)
    });
}

#[inline]
fn remove_middle_make_bit_string(len: usize) -> BitString {
    (0..len).map(remove_middle_bit_at).collect()
}

fn remove_middle_make_string(len: usize) -> String {
    let mut out = String::with_capacity(len);
    for index in 0..len {
        out.push(remove_middle_bit_char(remove_middle_bit_at(index)));
    }
    out
}

#[inline]
fn remove_middle_bit_char(value: bool) -> char {
    if value { '1' } else { '0' }
}

#[inline]
fn remove_middle_bit_at(index: usize) -> bool {
    remove_middle_mix64(index as u64) & 1 != 0
}

#[inline]
fn remove_middle_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
