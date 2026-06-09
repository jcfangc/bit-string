use core::fmt;

use super::*;

impl fmt::Display for BitString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for index in 0..self.len {
            f.write_str(if self.get(index).unwrap() { "1" } else { "0" })?;
        }

        Ok(())
    }
}

impl fmt::Debug for BitString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BitString(\"")?;
        fmt::Display::fmt(self, f)?;
        f.write_str("\")")
    }
}
