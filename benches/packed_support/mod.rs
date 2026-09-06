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

pub(crate) const WIDTHS: &[u8] = &[1, 2, 3, 4, 7, 8];
pub(crate) const LENGTHS: &[usize] = &[16, 64, 1_024, 65_536, 1_048_576];

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
