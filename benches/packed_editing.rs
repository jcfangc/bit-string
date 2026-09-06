#[path = "packed_support/mod.rs"]
mod support;

use divan::{Bencher, black_box};
use support::{Code, codes, packed};

fn main() {
    divan::main();
}

fn set_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = len / 2;
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let previous = value.set(index, Code(0));
        black_box(&*value);
        black_box(previous)
    });
}

fn set_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = len / 2;
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

fn push_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
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
    let index = len / 2;
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.insert(index, Code(0));
        black_box(&*value);
        black_box(value.char_len())
    });
}

fn insert_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = len / 2;
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        value.insert(index, 0);
        black_box(&*value);
        black_box(value.len())
    });
}

fn remove_middle_packed<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = packed::<BITS>(&codes(BITS, len));
    let index = len / 2;
    bencher.with_inputs(|| input.clone()).bench_refs(|value| {
        let removed = value.remove(index);
        black_box(&*value);
        black_box(removed)
    });
}

fn remove_middle_vec<const BITS: u8>(bencher: Bencher, len: usize) {
    let input = codes(BITS, len);
    let index = len / 2;
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

            pair!("set_middle", set_middle_packed, set_middle_vec);
            pair!("push", push_packed, push_vec);
            pair!("pop", pop_packed, pop_vec);
            pair!("insert_middle", insert_middle_packed, insert_middle_vec);
            pair!("remove_middle", remove_middle_packed, remove_middle_vec);
            pair!("extend", extend_packed, extend_vec);
        }
    };
}

crate::for_each_packed_case!(define_case);
