/// An enum representing the endianness used in a layout for accessing primitive integer fields.
pub enum EndianKind {
    Big,
    Little,
}

/// This marker trait represents the endianness used in a layout for accessing primitive integer fields.
pub trait Endianness {
    /// Accessor to the endianness as a const value
    const KIND: EndianKind;
}

/// This is a marker type to mark layouts using big endian encoding. The alternative is [LittleEndian] encoding.
///
/// # Example
/// ```
/// use binary_layout::prelude::*;
///
/// define_layout!(my_layout, BigEndian, {
///   field1: i16,
///   field2: u32,
/// });
/// ```
pub struct BigEndian {}
impl Endianness for BigEndian {
    const KIND: EndianKind = EndianKind::Big;
}

/// This is a marker type to mark layouts using little endian encoding. The alternative is [BigEndian] encoding.
///
/// # Example
/// ```
/// use binary_layout::prelude::*;
///
/// define_layout!(my_layout, LittleEndian, {
///   field1: i16,
///   field2: u32,
/// });
/// ```
pub struct LittleEndian {}
impl Endianness for LittleEndian {
    const KIND: EndianKind = EndianKind::Little;
}
