/// A character with a fixed-width bit representation.
///
/// This trait is intended for fieldless `#[repr(u8)]` enums. Implementations
/// must satisfy all of the following invariants:
///
/// - [`BITS`](Self::BITS) does not exceed eight;
/// - [`code`](Self::code) fits in `BITS` bits;
/// - `from_code(code(value)) == Some(value)`;
/// - each valid code identifies at most one value.
///
/// [`PackedString`](crate::PackedString) checks the first three invariants
/// whenever it writes a value. No `transmute` is used.
pub trait PackedChar: Copy + Eq {
    /// Number of bits occupied by one character.
    const BITS: u8;

    /// Returns the character's packed value.
    fn code(self) -> u8;

    /// Decodes a packed value, rejecting unused bit patterns.
    fn from_code(code: u8) -> Option<Self>;
}
