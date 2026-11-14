#[derive(Clone, Copy)]
enum AnyFixture {
    Zero,
    OneAt(usize),
}

#[derive(Clone, Copy)]
enum AllFixture {
    One,
    ZeroAt(usize),
}

macro_rules! define_any_case {
    ($old:ident, $scan:ident, $case:literal, $len:expr, $fixture:expr) => {
        #[divan::bench(name = concat!("any/", $case, "/ours_count_ones"))]
        fn $old(bencher: Bencher) {
            let bits = make_any_fixture($len, $fixture);
            assert_eq!(bits.any(), early_any(&bits));
            bencher.bench(|| black_box(&bits).any());
        }

        #[divan::bench(name = concat!("any/", $case, "/ours_word_scan"))]
        fn $scan(bencher: Bencher) {
            let bits = make_any_fixture($len, $fixture);
            assert_eq!(bits.any(), early_any(&bits));
            bencher.bench(|| early_any(black_box(&bits)));
        }
    };
}

macro_rules! define_all_case {
    ($old:ident, $scan:ident, $case:literal, $len:expr, $fixture:expr) => {
        #[divan::bench(name = concat!("all/", $case, "/ours_count_ones"))]
        fn $old(bencher: Bencher) {
            let bits = make_all_fixture($len, $fixture);
            assert_eq!(bits.all(), early_all(&bits));
            bencher.bench(|| black_box(&bits).all());
        }

        #[divan::bench(name = concat!("all/", $case, "/ours_word_scan"))]
        fn $scan(bencher: Bencher) {
            let bits = make_all_fixture($len, $fixture);
            assert_eq!(bits.all(), early_all(&bits));
            bencher.bench(|| early_all(black_box(&bits)));
        }
    };
}

define_any_case!(any_65_zero_old, any_65_zero_scan, "len_65/all_zero", 65, AnyFixture::Zero);
define_any_case!(any_65_first_old, any_65_first_scan, "len_65/first_one", 65, AnyFixture::OneAt(0));
define_any_case!(any_65_middle_old, any_65_middle_scan, "len_65/middle_one", 65, AnyFixture::OneAt(32));
define_any_case!(any_65_last_old, any_65_last_scan, "len_65/last_one", 65, AnyFixture::OneAt(64));
define_any_case!(any_large_zero_old, any_large_zero_scan, "len_65536/all_zero", 65_536, AnyFixture::Zero);
define_any_case!(any_large_first_old, any_large_first_scan, "len_65536/first_one", 65_536, AnyFixture::OneAt(0));
define_any_case!(any_large_middle_old, any_large_middle_scan, "len_65536/middle_one", 65_536, AnyFixture::OneAt(32_768));
define_any_case!(any_large_last_old, any_large_last_scan, "len_65536/last_one", 65_536, AnyFixture::OneAt(65_535));

define_all_case!(all_65_one_old, all_65_one_scan, "len_65/all_one", 65, AllFixture::One);
define_all_case!(all_65_first_old, all_65_first_scan, "len_65/first_zero", 65, AllFixture::ZeroAt(0));
define_all_case!(all_65_middle_old, all_65_middle_scan, "len_65/middle_zero", 65, AllFixture::ZeroAt(32));
define_all_case!(all_65_last_old, all_65_last_scan, "len_65/last_zero", 65, AllFixture::ZeroAt(64));
define_all_case!(all_large_one_old, all_large_one_scan, "len_65536/all_one", 65_536, AllFixture::One);
define_all_case!(all_large_first_old, all_large_first_scan, "len_65536/first_zero", 65_536, AllFixture::ZeroAt(0));
define_all_case!(all_large_middle_old, all_large_middle_scan, "len_65536/middle_zero", 65_536, AllFixture::ZeroAt(32_768));
define_all_case!(all_large_last_old, all_large_last_scan, "len_65536/last_zero", 65_536, AllFixture::ZeroAt(65_535));

fn make_any_fixture(len: usize, fixture: AnyFixture) -> BitString {
    let mut bits = BitString::zeros(len);
    if let AnyFixture::OneAt(index) = fixture {
        bits.set(index, true);
    }
    bits
}

fn make_all_fixture(len: usize, fixture: AllFixture) -> BitString {
    let mut bits = BitString::ones(len);
    if let AllFixture::ZeroAt(index) = fixture {
        bits.set(index, false);
    }
    bits
}

fn early_any(bits: &BitString) -> bool {
    let full_word_len = bits.bit_len() / 64;
    if bits.words()[..full_word_len].iter().any(|&word| word != 0) {
        return true;
    }

    let tail_len = bits.bit_len() % 64;
    tail_len != 0 && bits.words()[full_word_len] & low_mask(tail_len) != 0
}

fn early_all(bits: &BitString) -> bool {
    let full_word_len = bits.bit_len() / 64;
    if bits.words()[..full_word_len]
        .iter()
        .any(|&word| word != u64::MAX)
    {
        return false;
    }

    let tail_len = bits.bit_len() % 64;
    tail_len == 0 || bits.words()[full_word_len] & low_mask(tail_len) == low_mask(tail_len)
}

fn low_mask(bit_len: usize) -> u64 {
    (1u64 << bit_len) - 1
}
