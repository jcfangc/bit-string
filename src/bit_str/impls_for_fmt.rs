use core::fmt;

use crate::BitStr;

const DISPLAY_CHUNK_LEN: usize = crate::WORD_BITS;

impl fmt::Display for BitStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = [0u8; DISPLAY_CHUNK_LEN];

        for bit_start in (0..self.bit_len).step_by(DISPLAY_CHUNK_LEN) {
            let chunk_len = (self.bit_len - bit_start).min(DISPLAY_CHUNK_LEN);
            let mut bits = self.get_chunk(bit_start);

            for output_index in 0..chunk_len {
                output[output_index] = b'0' + (bits & 1) as u8;
                bits >>= 1;
            }

            // SAFETY: Every byte written above is either b'0' or b'1'.
            let chunk = unsafe { core::str::from_utf8_unchecked(&output[..chunk_len]) };
            f.write_str(chunk)?;
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
