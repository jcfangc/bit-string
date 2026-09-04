/// A character with a fixed-width bit representation.
///
/// This trait is intended for fieldless `#[repr(u8)]` enums. Use the
/// `#[packed(bits = N)]` attribute for that common case; manual
/// implementations must satisfy all of the following invariants:
///
/// - `BITS` does not exceed eight;
/// - [`code`](Self::code) fits in `BITS` bits;
/// - `from_code(code(value)) == Some(value)`;
/// - each valid code identifies at most one value.
///
/// [`PackedString`](crate::PackedString) relies on these invariants. The packed
/// attribute validates them at compile time and generates both directions from
/// the same discriminants. No `transmute` is used.
pub trait PackedChar<const BITS: u8>: Copy + Eq {
    /// Returns the character's packed value.
    fn code(self) -> u8;

    /// Decodes a packed value, rejecting unused bit patterns.
    fn from_code(code: u8) -> Option<Self>;
}
