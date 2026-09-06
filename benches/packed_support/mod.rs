#![allow(dead_code)]

use bit_string::{PackedString, traits::PackedChar};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Code(pub(crate) u8);

impl<const BITS: u8> PackedChar<BITS> for Code {
    fn code(self) -> u8 {
        self.0
    }

    fn from_code(code: u8) -> Option<Self> {
        let limit = if BITS == 8 { 256 } else { 1 << BITS };
        (u16::from(code) < limit).then_some(Self(code))
    }
}

#[macro_export]
macro_rules! for_each_packed_case {
    ($macro:ident) => {
        $macro!(bits_1_len_16, 1, 16);
        $macro!(bits_1_len_64, 1, 64);
        $macro!(bits_1_len_1024, 1, 1_024);
        $macro!(bits_1_len_65536, 1, 65_536);
        $macro!(bits_1_len_1048576, 1, 1_048_576);
        $macro!(bits_2_len_16, 2, 16);
        $macro!(bits_2_len_64, 2, 64);
        $macro!(bits_2_len_1024, 2, 1_024);
        $macro!(bits_2_len_65536, 2, 65_536);
        $macro!(bits_2_len_1048576, 2, 1_048_576);
        $macro!(bits_3_len_16, 3, 16);
        $macro!(bits_3_len_64, 3, 64);
        $macro!(bits_3_len_1024, 3, 1_024);
        $macro!(bits_3_len_65536, 3, 65_536);
        $macro!(bits_3_len_1048576, 3, 1_048_576);
        $macro!(bits_4_len_16, 4, 16);
        $macro!(bits_4_len_64, 4, 64);
        $macro!(bits_4_len_1024, 4, 1_024);
        $macro!(bits_4_len_65536, 4, 65_536);
        $macro!(bits_4_len_1048576, 4, 1_048_576);
        $macro!(bits_7_len_16, 7, 16);
        $macro!(bits_7_len_64, 7, 64);
        $macro!(bits_7_len_1024, 7, 1_024);
        $macro!(bits_7_len_65536, 7, 65_536);
        $macro!(bits_7_len_1048576, 7, 1_048_576);
        $macro!(bits_8_len_16, 8, 16);
        $macro!(bits_8_len_64, 8, 64);
        $macro!(bits_8_len_1024, 8, 1_024);
        $macro!(bits_8_len_65536, 8, 65_536);
        $macro!(bits_8_len_1048576, 8, 1_048_576);
    };
}

#[macro_export]
macro_rules! for_each_grit_case {
    ($macro:ident) => {
        $macro!(bits_1_len_16, 1, 16, grit_bitvec::u8_as_u1);
        $macro!(bits_1_len_64, 1, 64, grit_bitvec::u8_as_u1);
        $macro!(bits_1_len_1024, 1, 1_024, grit_bitvec::u8_as_u1);
        $macro!(bits_1_len_65536, 1, 65_536, grit_bitvec::u8_as_u1);
        $macro!(bits_1_len_1048576, 1, 1_048_576, grit_bitvec::u8_as_u1);
        $macro!(bits_2_len_16, 2, 16, grit_bitvec::u8_as_u2);
        $macro!(bits_2_len_64, 2, 64, grit_bitvec::u8_as_u2);
        $macro!(bits_2_len_1024, 2, 1_024, grit_bitvec::u8_as_u2);
        $macro!(bits_2_len_65536, 2, 65_536, grit_bitvec::u8_as_u2);
        $macro!(bits_2_len_1048576, 2, 1_048_576, grit_bitvec::u8_as_u2);
        $macro!(bits_3_len_16, 3, 16, grit_bitvec::u8_as_u3);
        $macro!(bits_3_len_64, 3, 64, grit_bitvec::u8_as_u3);
        $macro!(bits_3_len_1024, 3, 1_024, grit_bitvec::u8_as_u3);
        $macro!(bits_3_len_65536, 3, 65_536, grit_bitvec::u8_as_u3);
        $macro!(bits_3_len_1048576, 3, 1_048_576, grit_bitvec::u8_as_u3);
        $macro!(bits_4_len_16, 4, 16, grit_bitvec::u8_as_u4);
        $macro!(bits_4_len_64, 4, 64, grit_bitvec::u8_as_u4);
        $macro!(bits_4_len_1024, 4, 1_024, grit_bitvec::u8_as_u4);
        $macro!(bits_4_len_65536, 4, 65_536, grit_bitvec::u8_as_u4);
        $macro!(bits_4_len_1048576, 4, 1_048_576, grit_bitvec::u8_as_u4);
        $macro!(bits_7_len_16, 7, 16, grit_bitvec::u8_as_u7);
        $macro!(bits_7_len_64, 7, 64, grit_bitvec::u8_as_u7);
        $macro!(bits_7_len_1024, 7, 1_024, grit_bitvec::u8_as_u7);
        $macro!(bits_7_len_65536, 7, 65_536, grit_bitvec::u8_as_u7);
        $macro!(bits_7_len_1048576, 7, 1_048_576, grit_bitvec::u8_as_u7);
    };
}

pub(crate) fn codes(bits: u8, len: usize) -> Vec<u8> {
    let mask = if bits == 8 {
        u8::MAX
    } else {
        ((1u16 << bits) - 1) as u8
    };

    (0..len)
        .map(|index| (mix64(index as u64) as u8) & mask)
        .collect()
}

pub(crate) fn packed<const BITS: u8>(codes: &[u8]) -> PackedString<Code, BITS> {
    PackedString::from_chars(codes.iter().copied().map(Code))
}

pub(crate) fn indices(len: usize) -> Vec<usize> {
    (0..len.max(1))
        .map(|index| (mix64(index as u64 ^ 0x517c_c1b7_2722_0a95) as usize) % len)
        .collect()
}

pub(crate) fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
