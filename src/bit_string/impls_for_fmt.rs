use core::fmt;

use super::*;

impl fmt::Display for BitString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_bit_str(), f)
    }
}

impl fmt::Debug for BitString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BitString(\"")?;
        fmt::Display::fmt(self, f)?;
        f.write_str("\")")
    }
}
