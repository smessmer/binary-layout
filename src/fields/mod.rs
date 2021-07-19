use super::endianness::Endianness;

pub mod primitive;
pub mod wrapped;

// TODO Have implementations of trait functions inherit docs from trait instead of copy&pasting them, is there a crate for this?

/// This trait offers access to the metadata of a field in a layout
pub trait IField {
    /// The endianness of the field
    type Endian: Endianness;

    /// The offset of the field in the layout.
    ///
    /// # Example
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   field1: u16,
    ///   field2: i32,
    ///   field3: u8,
    /// });
    ///
    /// assert_eq!(0, my_layout::field1::OFFSET);
    /// assert_eq!(2, my_layout::field2::OFFSET);
    /// assert_eq!(6, my_layout::field3::OFFSET);
    /// ```
    const OFFSET: usize;
}

/// This trait offers access to the metadata of a sized field in a layout.
/// Sized fields are all fields with a defined size. This is almost all fields.
/// The only exception is an unsized array field that can be used to match
/// tail data, i.e. any data at the end of the storage after all other fields
/// were defined and until the storage ends.
pub trait ISizedField: IField {
    /// The size of the field in the layout.
    ///
    /// # Example
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   field1: u16,
    ///   field2: i32,
    ///   field3: u8,
    /// });
    ///
    /// assert_eq!(2, my_layout::field1::SIZE);
    /// assert_eq!(4, my_layout::field2::SIZE);
    /// assert_eq!(1, my_layout::field3::SIZE);
    /// ```
    const SIZE: usize;
}

/// This trait is implemented for fields with "copy access",
/// i.e. fields that read/write data by copying it from the
/// binary blob. Examples of this are primitive types
/// like u8, i32, ...
pub trait IFieldCopyAccess: IField {
    /// The data type that is returned from read calls and has to be
    /// passed in to write calls. This can be different from the primitive
    /// type used in the binary blob, since that primitive type can be
    /// wrapped into a high level type before being returned from read
    /// calls (or vice versa unwrapped when writing).
    type HighLevelType;

    /// TODO Doc
    fn read(storage: &[u8]) -> Self::HighLevelType;
    /// TODO Doc
    fn write(storage: &mut [u8], v: Self::HighLevelType);
}

/// This trait si implemented for fields with "slice access",
/// i.e. fields that are read/write directly without a copy
/// by returning a borrowed slice to the underlying data.
pub trait IFieldSliceAccess<'a>: IField {
    /// TODO Doc
    type SliceType: 'a;
    /// TODO Doc
    type MutSliceType: 'a;

    /// TODO Doc
    fn data(storage: &'a [u8]) -> Self::SliceType;
    /// TODO Doc
    fn data_mut(storage: &'a mut [u8]) -> Self::MutSliceType;
}
