use core::marker::PhantomData;

use crate::data_types::DataTypeMetadata;
use crate::endianness::Endianness;
use crate::view::PrimitiveFieldView;
use crate::Field;

mod copy_access;
mod nested_access;
mod slice_access;
mod view;

pub use copy_access::{FieldCopyAccess, FieldReadExt, FieldWriteExt};
// pub use nested_access::{BorrowingNestedView, NestedViewInfo, OwningNestedView};
pub use slice_access::FieldSliceAccess;

/// A [PrimitiveField] is a [Field](crate::Field) that directly represents a primitive type like [u8], [i16], ...
/// See [Field](crate::Field) for more info on this API.
///
/// # Example:
/// ```
/// use binary_layout::prelude::*;
///
/// binary_layout!(my_layout, LittleEndian, {
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
pub struct PrimitiveField<T: ?Sized, E: Endianness, const OFFSET_: usize> {
    _p1: PhantomData<T>,
    _p2: PhantomData<E>,
}

impl<T: ?Sized + DataTypeMetadata, E: Endianness, const OFFSET_: usize> Field
    for PrimitiveField<T, E, OFFSET_>
{
    /// See [Field::Endian]
    type Endian = E;
    /// See [Field::View]
    type View<S> = <T as DataTypeMetadata>::View<S, Self>;
    /// See [Field::OFFSET]
    const OFFSET: usize = OFFSET_;
    /// See [Field::SIZE]
    const SIZE: Option<usize> = T::SIZE;
}
