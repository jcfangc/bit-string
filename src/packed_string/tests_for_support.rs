use core::fmt;

use crate::packed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[packed(bits = 2)]
pub(super) enum Letter {
    A = 0b00,
    B = 0b01,
    C = 0b10,
    D = 0b11,
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[packed(bits = 1)]
pub(super) enum Only {
    Value = 0,
}

pub(super) type LetterString = super::PackedString<Letter, 2>;
pub(super) type OnlyString = super::PackedString<Only, 1>;
