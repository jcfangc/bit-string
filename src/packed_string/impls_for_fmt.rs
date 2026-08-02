use core::fmt;

use super::*;

impl<C, const BITS: u8> fmt::Debug for PackedString<C, BITS>
where
    C: PackedChar<BITS> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<C, const BITS: u8> fmt::Display for PackedString<C, BITS>
where
    C: PackedChar<BITS> + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for character in self.iter() {
            fmt::Display::fmt(&character, f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests_for_fmt;
