use crate::WORD_BITS;

/// Word-by-word scan: for each shift, check every word pair in
/// `[0, word_limit)` for a matching window.
pub(super) fn scalar<F>(
    haystack: &[u64],
    needle_first: u64,
    needle_mask: u64,
    last_start: usize,
    word_limit: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    for shift in 0..WORD_BITS {
        for i in 0..word_limit {
            let pos = i * WORD_BITS + shift;
            if pos > last_start {
                break;
            }
            let window = if shift == 0 {
                haystack[i]
            } else {
                let w0 = haystack[i];
                let w1 = haystack.get(i + 1).copied().unwrap_or(0);
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            if (window & needle_mask) == needle_first && verify(pos) {
                return Some(pos);
            }
        }
    }
    None
}
