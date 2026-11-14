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
    ($reference:ident, $ours:ident, $case:literal, $len:expr, $fixture:expr) => {
        #[divan::bench(name = concat!("any/", $case, "/count_ones_reference"))]
        fn $reference(bencher: Bencher) {
            let bits = make_any_fixture($len, $fixture);
            assert_eq!(bits.any(), bits.count_ones() != 0);
            bencher.bench(|| black_box(&bits).count_ones() != 0);
        }

        #[divan::bench(name = concat!("any/", $case, "/ours_string"))]
        fn $ours(bencher: Bencher) {
            let bits = make_any_fixture($len, $fixture);
            assert_eq!(bits.any(), bits.count_ones() != 0);
            bencher.bench(|| black_box(&bits).any());
        }
    };
}

macro_rules! define_all_case {
    ($reference:ident, $ours:ident, $case:literal, $len:expr, $fixture:expr) => {
        #[divan::bench(name = concat!("all/", $case, "/count_ones_reference"))]
        fn $reference(bencher: Bencher) {
            let bits = make_all_fixture($len, $fixture);
            assert_eq!(bits.all(), bits.count_ones() == bits.bit_len());
            bencher.bench(|| {
                let bits = black_box(&bits);
                bits.count_ones() == bits.bit_len()
            });
        }

        #[divan::bench(name = concat!("all/", $case, "/ours_string"))]
        fn $ours(bencher: Bencher) {
            let bits = make_all_fixture($len, $fixture);
            assert_eq!(bits.all(), bits.count_ones() == bits.bit_len());
            bencher.bench(|| black_box(&bits).all());
        }
    };
}

macro_rules! define_grouped_any_case {
    ($four:ident, $eight:ident, $case:literal, $fixture:expr) => {
        #[divan::bench(name = concat!("any/len_65536/", $case, "/word_scan_4"))]
        fn $four(bencher: Bencher) {
            let bits = make_any_fixture(65_536, $fixture);
            assert_eq!(bits.any(), grouped_any::<4>(&bits));
            bencher.bench(|| grouped_any::<4>(black_box(&bits)));
        }

        #[divan::bench(name = concat!("any/len_65536/", $case, "/word_scan_8"))]
        fn $eight(bencher: Bencher) {
            let bits = make_any_fixture(65_536, $fixture);
            assert_eq!(bits.any(), grouped_any::<8>(&bits));
            bencher.bench(|| grouped_any::<8>(black_box(&bits)));
        }
    };
}

macro_rules! define_grouped_all_case {
    ($four:ident, $eight:ident, $case:literal, $fixture:expr) => {
        #[divan::bench(name = concat!("all/len_65536/", $case, "/word_scan_4"))]
        fn $four(bencher: Bencher) {
            let bits = make_all_fixture(65_536, $fixture);
            assert_eq!(bits.all(), grouped_all::<4>(&bits));
            bencher.bench(|| grouped_all::<4>(black_box(&bits)));
        }

        #[divan::bench(name = concat!("all/len_65536/", $case, "/word_scan_8"))]
        fn $eight(bencher: Bencher) {
            let bits = make_all_fixture(65_536, $fixture);
            assert_eq!(bits.all(), grouped_all::<8>(&bits));
            bencher.bench(|| grouped_all::<8>(black_box(&bits)));
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

define_grouped_any_case!(any_group4_zero, any_group8_zero, "all_zero", AnyFixture::Zero);
define_grouped_any_case!(any_group4_first, any_group8_first, "first_one", AnyFixture::OneAt(0));
define_grouped_any_case!(any_group4_middle, any_group8_middle, "middle_one", AnyFixture::OneAt(32_768));
define_grouped_any_case!(any_group4_last, any_group8_last, "last_one", AnyFixture::OneAt(65_535));

define_grouped_all_case!(all_group4_one, all_group8_one, "all_one", AllFixture::One);
define_grouped_all_case!(all_group4_first, all_group8_first, "first_zero", AllFixture::ZeroAt(0));
define_grouped_all_case!(all_group4_middle, all_group8_middle, "middle_zero", AllFixture::ZeroAt(32_768));
define_grouped_all_case!(all_group4_last, all_group8_last, "last_zero", AllFixture::ZeroAt(65_535));

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

fn grouped_any<const GROUP: usize>(bits: &BitString) -> bool {
    let words = bits.words();
    let full_word_len = bits.bit_len() / 64;
    let mut word_index = 0;

    while word_index + GROUP <= full_word_len {
        let mut combined = 0;
        for word in &words[word_index..word_index + GROUP] {
            combined |= word;
        }
        if combined != 0 {
            return true;
        }
        word_index += GROUP;
    }

    if words[word_index..full_word_len]
        .iter()
        .any(|&word| word != 0)
    {
        return true;
    }

    let tail_len = bits.bit_len() % 64;
    tail_len != 0 && words[full_word_len] & low_mask(tail_len) != 0
}

fn grouped_all<const GROUP: usize>(bits: &BitString) -> bool {
    let words = bits.words();
    let full_word_len = bits.bit_len() / 64;
    let mut word_index = 0;

    while word_index + GROUP <= full_word_len {
        let mut combined = u64::MAX;
        for word in &words[word_index..word_index + GROUP] {
            combined &= word;
        }
        if combined != u64::MAX {
            return false;
        }
        word_index += GROUP;
    }

    if words[word_index..full_word_len]
        .iter()
        .any(|&word| word != u64::MAX)
    {
        return false;
    }

    let tail_len = bits.bit_len() % 64;
    tail_len == 0 || words[full_word_len] == low_mask(tail_len)
}

fn low_mask(bit_len: usize) -> u64 {
    (1u64 << bit_len) - 1
}
