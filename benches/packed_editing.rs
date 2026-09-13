#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{Code, aligned_index, codes, cross_word_index, packed, unaligned_index};

fn main() {
    divan::main();
}

fn set_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = aligned_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let previous = value.set(index, Code(0));
        black_box(&*value);
        black_box(previous)
    });
}

fn set_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = aligned_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let old = std::mem::replace(&mut value[index], 0);
        black_box(&*value);
        black_box(old)
    });
}

fn push_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.push(Code(0));
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn push_amortized_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len - 1));
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.push(Code(0));
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn push_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.push(0);
        black_box(&*value);
        black_box(value.len())
    });
}

fn push_amortized_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len - 1);
    bencher
        .with_inputs(|| {
            let mut value = input.clone();
            value.reserve(1);
            value
        })
        .bench_refs(|value| {
            value.push(0);
            black_box(&*value);
            black_box(value.len())
        });
}

fn pop_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let popped = value.pop();
        black_box(&*value);
        black_box(popped)
    });
}

fn pop_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let popped = value.pop();
        black_box(&*value);
        black_box(popped)
    });
}

fn insert_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = unaligned_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.insert(index, Code(0));
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn insert_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = unaligned_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.insert(index, 0);
        black_box(&*value);
        black_box(value.len())
    });
}

fn remove_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = cross_word_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let removed = value.remove(index);
        black_box(&*value);
        black_box(removed)
    });
}

fn remove_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = cross_word_index::<BITS>(len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let removed = value.remove(index);
        black_box(&*value);
        black_box(removed)
    });
}

fn extend_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.extend(extension.iter().copied().map(Code));
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn extend_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let extension = codes(BITS, len / 4);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.extend(extension.iter().copied());
        black_box(&*value);
        black_box(value.len())
    });
}

macro_rules! define_case {
    ($case:ident, $bits:literal, $len:literal) => {
        mod $case {
            use super::*;

            macro_rules! pair {
                        ($scenario:literal, $packed:ident, $vec:ident) => {
                            #[divan::bench(
                                                                        name = concat!(
                                                                            "packed_editing/",
                                                                            $scenario,
                                                                            "/",
                                                                            stringify!($case),
                                                                            "/ours_packed_string"
                                                                        )
                                                                    )]
                            fn $packed(bencher: Bencher) {
                                crate::$packed::<$bits>(bencher, $len);
                            }

                            #[divan::bench(
                                                                        name = concat!(
                                                                            "packed_editing/",
                                                                            $scenario,
                                                                            "/",
                                                                            stringify!($case),
                                                                            "/vec_u8"
                                                                        )
                                                                    )]
                            fn $vec(bencher: Bencher) {
                                crate::$vec::<$bits>(bencher, $len);
                            }
                        };
                    }

            pair!("set_position/aligned", set_middle_packed, set_middle_vec);
            pair!("push_boundary_probe", push_packed, push_vec);
            pair!("push_amortized", push_amortized_packed, push_amortized_vec);
            pair!("pop", pop_packed, pop_vec);
            pair!("insert_position/unaligned", insert_middle_packed, insert_middle_vec);
            pair!("remove_position/word_boundary_or_fallback", remove_middle_packed, remove_middle_vec);
            pair!("extend_boundary_probe", extend_packed, extend_vec);
        }
    };
}

crate::for_each_packed_bench_case!(define_case);
