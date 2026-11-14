#[divan::bench(name = "insert/grow_word_len_64_no_spare/ours_bit_string")]
fn insert_grow_word_len_64_no_spare(bencher: Bencher) {
    insert_grow_word(bencher, 64, false);
}

#[divan::bench(name = "insert/grow_word_len_64_spare/ours_bit_string")]
fn insert_grow_word_len_64_spare(bencher: Bencher) {
    insert_grow_word(bencher, 64, true);
}

#[divan::bench(name = "insert/grow_word_len_65536_no_spare/ours_bit_string")]
fn insert_grow_word_len_65536_no_spare(bencher: Bencher) {
    insert_grow_word(bencher, 65_536, false);
}

#[divan::bench(name = "insert/grow_word_len_65536_spare/ours_bit_string")]
fn insert_grow_word_len_65536_spare(bencher: Bencher) {
    insert_grow_word(bencher, 65_536, true);
}

#[divan::bench(name = "remove/shrink_word_len_65/ours_bit_string")]
fn remove_shrink_word_len_65(bencher: Bencher) {
    remove_shrink_word(bencher, 65);
}

#[divan::bench(name = "remove/shrink_word_len_65537/ours_bit_string")]
fn remove_shrink_word_len_65537(bencher: Bencher) {
    remove_shrink_word(bencher, 65_537);
}

fn insert_grow_word(bencher: Bencher, len: usize, spare_word: bool) {
    let index = len / 2;

    bencher
        .with_inputs(|| make_insert_input(len, spare_word))
        .bench_refs(|bits| {
            bits.insert(black_box(index), true);
            black_box(bits);
        });
}

fn remove_shrink_word(bencher: Bencher, len: usize) {
    let index = len / 2;

    bencher
        .with_inputs(|| make_bits(len))
        .bench_refs(|bits| {
            black_box(bits.remove(black_box(index)));
            black_box(bits);
        });
}

fn make_insert_input(len: usize, spare_word: bool) -> BitString {
    let stored_len = len + usize::from(spare_word) * 64;
    let mut bits = make_bits(stored_len);
    bits.truncate(len);
    bits
}

fn make_bits(len: usize) -> BitString {
    (0..len).map(|index| insert_remove_bit_at(index)).collect()
}

fn insert_remove_bit_at(index: usize) -> bool {
    let mut value = (index as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (value ^ (value >> 31)) & 1 != 0
}
