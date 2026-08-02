use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Broken;

impl PackedChar for Broken {
    const BITS: u8 = 1;

    fn code(self) -> u8 {
        2
    }

    fn from_code(_: u8) -> Option<Self> {
        Some(Self)
    }
}

#[test]
#[should_panic(expected = "does not fit")]
fn rejects_a_code_that_does_not_fit() {
    let _ = checked_code(Broken);
}
