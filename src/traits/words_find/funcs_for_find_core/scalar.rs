use crate::WORD_BITS;

pub(super) fn scalar_find<F>(
    haystack: &[u64],
    needle_first: u64,
    needle_mask: u64,
    last_start: usize,
    verify: &mut F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    for i in 0..haystack.len() {
        let w0 = haystack[i];
        let w1 = haystack.get(i + 1).copied().unwrap_or(0);
        for shift in 0..WORD_BITS {
            let pos = i * WORD_BITS + shift;
            if pos > last_start {
                break;
            }
            let window = if shift == 0 {
                w0
            } else {
                (w0 >> shift) | (w1 << (WORD_BITS - shift))
            };
            if (window & needle_mask) == needle_first && verify(pos) {
                return Some(pos);
            }
        }
    }
    None
}
