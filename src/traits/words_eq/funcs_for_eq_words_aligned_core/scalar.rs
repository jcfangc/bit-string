pub(super) fn eq_words(src: &[u64], other: &[u64], count: usize) -> bool {
    for i in 0..count {
        if src[i] != other[i] {
            return false;
        }
    }
    true
}
