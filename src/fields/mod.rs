use super::endianness::Endianness;

pub mod primitive;
pub mod wrapped;

///
/// A field represents one of the fields in the data layout and offers accessors
/// for it. It remembers the offset of the field in its const generic parameter
/// and the accessors use that to access the field.
///
/// A field does not hold any data storage, so if you use this API directly, you have to pass in
/// the storage pointer for each call. If you want an API object that remembers the storage,
/// take a look at the [FieldView](crate::FieldView) based API instead.
///
/// By itself, [Field] only offers the things common to all fields, but there
/// are additional traits for fields that fulfill certain properties:
/// - [SizedField] for fields that have a defined size (most types except for open ended byte arrays)
/// - [FieldCopyAccess] for fields that read/write data by copying it to/from the storage. This includes primitive types like [i8] or [u16].
///   This trait offers [FieldCopyAccess::read] and [FieldCopyAccess::write] to read or write such fields.
/// - [FieldSliceAccess] for fields that read/write data by creating sub-slices over the storage. This includes, for example, byte arrays
///   and this trait offers [FieldSliceAccess::data] and [FieldSliceAccess::data_mut] to access such fields.
///
/// # Example:
/// ```
/// use binary_layout::prelude::*;
///
/// define_layout!(my_layout, LittleEndian, {
///   field_one: u16,
///   another_field: [u8; 16],
///   something_else: u32,
///   tail_data: [u8],
/// });
///
/// fn func(storage_data: &mut [u8]) {
///   // read some data
///   let format_version_header: u16 = my_layout::field_one::read(storage_data);
///   // equivalent: let format_version_header = u16::from_le_bytes((&storage_data[0..2]).try_into().unwrap());
///
///   // write some data
///   my_layout::something_else::write(storage_data, 10);
///   // equivalent: data_slice[18..22].copy_from_slice(&10u32.to_le_bytes());
///
///   // access a data region
///   let tail_data: &[u8] = my_layout::tail_data::data(storage_data);
///   // equivalent: let tail_data: &[u8] = &data_slice[22..];
///
///   // and modify it
///   my_layout::tail_data::data_mut(storage_data)[..5].copy_from_slice(&[1, 2, 3, 4, 5]);
///   // equivalent: data_slice[18..22].copy_from_slice(&[1, 2, 3, 4, 5]);
/// }
/// ```
pub trait Field {
    /// The endianness of the field. Can be [LittleEndian](crate::LittleEndian) or [BigEndian](crate::BigEndian).
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
pub trait SizedField: Field {
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
/// i.e. fields that read/write data by copying it from/to the
/// binary blob. Examples of this are primitive types
/// like u8, i32, ...
pub trait FieldCopyAccess: Field {
    /// The data type that is returned from read calls and has to be
    /// passed in to write calls. This can be different from the primitive
    /// type used in the binary blob, since that primitive type can be
    /// wrapped (see [crate::WrappedField] ) into a high level type before being returned from read
    /// calls (or vice versa unwrapped when writing).
    type HighLevelType;

    /// Read the field from a given data region, assuming the defined layout, using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   //... other fields ...
    ///   some_integer_field: u16,
    ///   //... other fields ...
    /// });
    ///
    /// fn func(storage_data: &[u8]) {
    ///   let read: u16 = my_layout::some_integer_field::read(storage_data);
    /// }
    /// ```
    fn read(storage: &[u8]) -> Self::HighLevelType;

    /// Write the field to a given data region, assuming the defined layout, using the [Field] API.
    ///
    /// # Example:
    ///         
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   //... other fields ...
    ///   some_integer_field: u16,
    ///   //... other fields ...
    /// });
    ///
    /// fn func(storage_data: &mut [u8]) {
    ///   my_layout::some_integer_field::write(storage_data, 10);
    /// }
    /// ```
    fn write(storage: &mut [u8], v: Self::HighLevelType);
}

/// This trait is implemented for fields with "slice access",
/// i.e. fields that are read/write directly without a copy
/// by returning a borrowed slice to the underlying data.
pub trait FieldSliceAccess<'a>: Field {
    /// The type of slice returned from calls requesting read access
    type SliceType: 'a;
    /// The type of slice returned from calls requesting write access
    type MutSliceType: 'a;

    /// Borrow the data in the byte array with read access using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &[u8]) {
    ///     let tail_data: &[u8] = my_layout::tail_data::data(storage_data);
    /// }
    /// ```
    fn data(storage: &'a [u8]) -> Self::SliceType;

    /// Borrow the data in the byte array with write access using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///     
    /// define_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &mut [u8]) {
    ///     let tail_data: &mut [u8] = my_layout::tail_data::data_mut(storage_data);
    /// }
    /// ```
    fn data_mut(storage: &'a mut [u8]) -> Self::MutSliceType;
}
