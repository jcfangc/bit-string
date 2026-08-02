use core::fmt;

use super::*;

impl<C: PackedChar + fmt::Debug> fmt::Debug for PackedString<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<C: PackedChar + fmt::Display> fmt::Display for PackedString<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for character in self.iter() {
            fmt::Display::fmt(&character, f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests_for_fmt;
