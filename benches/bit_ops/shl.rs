

#[divan::bench(name = "shl/len_65/amount_1/ours_string")]
fn shl_len_65_amount_1_bit_string(bencher: Bencher) {
    shl_bench_bit_string(bencher, 65, 1);
}

#[divan::bench(name = "shl/len_4096/amount_1/ours_string")]
fn shl_len_4096_amount_1_bit_string(bencher: Bencher) {
    shl_bench_bit_string(bencher, 4096, 1);
}

#[divan::bench(name = "shl/len_4096/amount_65/ours_string")]
fn shl_len_4096_amount_65_bit_string(bencher: Bencher) {
    shl_bench_bit_string(bencher, 4096, 65);
}

#[divan::bench(name = "shl/len_65536/amount_1/ours_string")]
fn shl_len_65536_amount_1_bit_string(bencher: Bencher) {
    shl_bench_bit_string(bencher, 65_536, 1);
}

#[divan::bench(name = "shl/len_65536/amount_65/ours_string")]
fn shl_len_65536_amount_65_bit_string(bencher: Bencher) {
    shl_bench_bit_string(bencher, 65_536, 65);
}

fn shl_bench_bit_string(bencher: Bencher, len: usize, amount: usize) {
    let bits = shl_make_bit_string(len);

    bencher.bench(|| black_box(&bits).shl(black_box(amount)));
}

#[inline]
fn shl_make_bit_string(len: usize) -> BitString {
    (0..len).map(|index| shl_mix64(index as u64) & 1 != 0).collect()
}

#[inline]
fn shl_mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
