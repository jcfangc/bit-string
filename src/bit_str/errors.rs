use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitStrRangeError {
    pub source_len: usize,
    pub requested_start: usize,
    pub requested_len: usize,
}

impl fmt::Display for BitStrRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BitStr range out of bounds: source has {} bits, \
             requested start={}, len={}",
            self.source_len, self.requested_start, self.requested_len,
        )
    }
}
