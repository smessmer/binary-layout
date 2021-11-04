use std::convert::TryFrom;

use super::PrimitiveField;
use crate::endianness::Endianness;
use crate::{Field, SizedField};

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

/// Field type `[u8]`:
/// This field represents an [open ended byte array](crate#open-ended-byte-arrays-u8).
/// In this impl, we define accessors for such fields.
impl<'a, E: Endianness, const OFFSET_: usize> FieldSliceAccess<'a>
    for PrimitiveField<[u8], E, OFFSET_>
{
    type SliceType = &'a [u8];
    type MutSliceType = &'a mut [u8];

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
    fn data(storage: &'a [u8]) -> &'a [u8] {
        &storage[Self::OFFSET..]
    }

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
    fn data_mut(storage: &'a mut [u8]) -> &'a mut [u8] {
        &mut storage[Self::OFFSET..]
    }
}

/// Field type `[u8; N]`:
/// This field represents a [fixed size byte array](crate#fixed-size-byte-arrays-u8-n).
/// In this impl, we define accessors for such fields.
impl<'a, E: Endianness, const N: usize, const OFFSET_: usize> FieldSliceAccess<'a>
    for PrimitiveField<[u8; N], E, OFFSET_>
{
    type SliceType = &'a [u8; N];
    type MutSliceType = &'a mut [u8; N];

    /// Borrow the data in the byte array with read access using the [Field] API.
    /// See also [FieldSliceAccess::data].
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     some_field: [u8; 5],
    ///     //... other fields
    /// });
    ///
    /// fn func(storage_data: &[u8]) {
    ///     let some_field: &[u8; 5] = my_layout::some_field::data(storage_data);
    /// }
    /// ```
    fn data(storage: &'a [u8]) -> &'a [u8; N] {
        <&[u8; N]>::try_from(&storage[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
    }

    /// Borrow the data in the byte array with write access using the [Field] API.
    /// See also [FieldSliceAccess::data_mut]
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     some_field: [u8; 5],
    ///     //... other fields
    /// });
    ///
    /// fn func(storage_data: &mut [u8]) {
    ///     let some_field: &mut [u8; 5] = my_layout::some_field::data_mut(storage_data);
    /// }
    /// ```
    fn data_mut(storage: &'a mut [u8]) -> &'a mut [u8; N] {
        <&mut [u8; N]>::try_from(&mut storage[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
    }
}
impl<E: Endianness, const N: usize, const OFFSET_: usize> SizedField
    for PrimitiveField<[u8; N], E, OFFSET_>
{
    /// See [SizedField::SIZE]
    const SIZE: usize = N;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_slice() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<[u8], LittleEndian, 5>;
        type Field2 = PrimitiveField<[u8], BigEndian, 7>;

        Field1::data_mut(&mut storage)[..5].copy_from_slice(&[10, 20, 30, 40, 50]);
        Field2::data_mut(&mut storage)[..5].copy_from_slice(&[60, 70, 80, 90, 100]);

        assert_eq!(&[10, 20, 60, 70, 80], &Field1::data(&storage)[..5]);
        assert_eq!(&[60, 70, 80, 90, 100], &Field2::data(&storage)[..5]);
    }

    #[test]
    fn test_array() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<[u8; 2], LittleEndian, 5>;
        type Field2 = PrimitiveField<[u8; 5], BigEndian, 6>;

        Field1::data_mut(&mut storage).copy_from_slice(&[10, 20]);
        Field2::data_mut(&mut storage).copy_from_slice(&[60, 70, 80, 90, 100]);

        assert_eq!(&[10, 60], Field1::data(&storage));
        assert_eq!(&[60, 70, 80, 90, 100], Field2::data(&storage));

        assert_eq!(2, PrimitiveField::<[u8; 2], LittleEndian, 5>::SIZE);
        assert_eq!(5, PrimitiveField::<[u8; 5], BigEndian, 5>::SIZE);
    }
}
