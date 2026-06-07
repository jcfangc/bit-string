use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseBitStringError {
    pub index: usize,
    pub byte: u8,
}

impl fmt::Display for ParseBitStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid bit character at byte index {}: {:?}",
            self.index, self.byte as char
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitStringLenMismatch {
    pub lhs_len: usize,
    pub rhs_len: usize,
}

impl fmt::Display for BitStringLenMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bit string length mismatch: lhs={}, rhs={}",
            self.lhs_len, self.rhs_len
        )
    }
}
