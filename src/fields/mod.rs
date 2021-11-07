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
pub trait SizedField: Field {
    /// The size of the field in the layout.
    /// This can be None if it is an open ended field like a byte slice
    ///
    /// # Example
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   field1: u16,
    ///   field2: i32,
    ///   field3: u8,
    ///   tail: [u8],
    /// });
    ///
    /// assert_eq!(Some(2), my_layout::field1::SIZE);
    /// assert_eq!(Some(4), my_layout::field2::SIZE);
    /// assert_eq!(Some(1), my_layout::field3::SIZE);
    /// assert_eq!(None, my_layout::tail::SIZE);
    /// ```
    const SIZE: Option<usize>;
}
