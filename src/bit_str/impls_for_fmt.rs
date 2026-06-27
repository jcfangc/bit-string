use core::fmt;

use crate::BitStr;

impl fmt::Display for BitStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for index in 0..self.bit_len {
            f.write_str(if self.get(index).unwrap() { "1" } else { "0" })?;
        }

        Ok(())
    }
}

impl fmt::Debug for BitStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BitStr(\"")?;
        fmt::Display::fmt(self, f)?;
        f.write_str("\")")
    }
}

#[cfg(test)]
mod tests_for_fmt;
