/// A character with a fixed-width bit representation.
///
/// This trait is intended for fieldless `#[repr(u8)]` enums. Implementations
/// must satisfy all of the following invariants:
///
/// - `BITS` does not exceed eight;
/// - [`code`](Self::code) fits in `BITS` bits;
/// - `from_code(code(value)) == Some(value)`;
/// - each valid code identifies at most one value.
///
/// [`PackedString`](crate::PackedString) validates `BITS` when it is
/// constructed, and checks the `code`/`from_code` agreement whenever it writes
/// a value. No `transmute` is used.
pub trait PackedChar<const BITS: u8>: Copy + Eq {
    /// Returns the character's packed value.
    fn code(self) -> u8;

    /// Decodes a packed value, rejecting unused bit patterns.
    fn from_code(code: u8) -> Option<Self>;
}
