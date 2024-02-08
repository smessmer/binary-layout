use super::super::Field;
use super::PrimitiveField;
use crate::data_types::{
    slice::{SliceRead, SliceWrite},
    DataTypeMetadata,
};
use crate::endianness::Endianness;

// TODO Split FieldSliceAccess into FieldDataAccess and FieldDataMutAccess

/// This trait is implemented for fields with "slice access",
/// i.e. fields that are read/write directly without a copy
/// by returning a borrowed slice to the underlying data.
pub trait FieldSliceAccess: Field {
    /// TODO Docs
    type HighLevelType: ?Sized;

    /// Borrow the data in the byte array with read access using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// binary_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &[u8]) {
    ///     let tail_data: &[u8] = my_layout::tail_data::data(storage_data);
    /// }
    /// ```
    fn data<'a>(storage: &'a [u8]) -> &'a Self::HighLevelType;

    /// Borrow the data in the byte array with write access using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// binary_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &mut [u8]) {
    ///     let tail_data: &mut [u8] = my_layout::tail_data::data_mut(storage_data);
    /// }
    /// ```
    fn data_mut<'a>(storage: &'a mut [u8]) -> &'a mut Self::HighLevelType;
}

impl<
        T: ?Sized + DataTypeMetadata + SliceRead + SliceWrite,
        E: Endianness,
        const OFFSET_: usize,
    > FieldSliceAccess for PrimitiveField<T, E, OFFSET_>
{
    type HighLevelType = T;

    /// Borrow the data in the byte array with read access using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// binary_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &[u8]) {
    ///     let tail_data: &[u8] = my_layout::tail_data::data(storage_data);
    /// }
    /// ```
    #[inline(always)]
    fn data<'a>(storage: &'a [u8]) -> &'a T {
        let storage = if let Some(size) = Self::SIZE {
            &storage[Self::OFFSET..(Self::OFFSET + size)]
        } else {
            &storage[Self::OFFSET..]
        };
        <T as SliceRead>::data(storage)
    }

    /// Borrow the data in the byte array with write access using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// binary_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &mut [u8]) {
    ///     let tail_data: &mut [u8] = my_layout::tail_data::data_mut(storage_data);
    /// }
    /// ```
    #[inline(always)]
    fn data_mut<'a>(storage: &'a mut [u8]) -> &'a mut T {
        let storage = if let Some(size) = Self::SIZE {
            &mut storage[Self::OFFSET..(Self::OFFSET + size)]
        } else {
            &mut storage[Self::OFFSET..]
        };
        <T as SliceWrite>::data_mut(storage)
    }
}

// /// Field type `[u8; N]`:
// /// This field represents a [fixed size byte array](crate#fixed-size-byte-arrays-u8-n).
// /// In this impl, we define accessors for such fields.
// impl<'a, E: Endianness, const N: usize, const OFFSET_: usize> FieldSliceAccess<'a>
//     for PrimitiveField<[u8; N], E, OFFSET_>
// {
//     type SliceType = &'a [u8; N];
//     type MutSliceType = &'a mut [u8; N];

//     /// Borrow the data in the byte array with read access using the [Field] API.
//     /// See also [FieldSliceAccess::data].
//     ///
//     /// # Example:
//     /// ```
//     /// use binary_layout::prelude::*;
//     ///
//     /// binary_layout!(my_layout, LittleEndian, {
//     ///     //... other fields ...
//     ///     some_field: [u8; 5],
//     ///     //... other fields
//     /// });
//     ///
//     /// fn func(storage_data: &[u8]) {
//     ///     let some_field: &[u8; 5] = my_layout::some_field::data(storage_data);
//     /// }
//     /// ```
//     #[inline(always)]
//     fn data(storage: &'a [u8]) -> &'a [u8; N] {
//         <&[u8; N]>::try_from(&storage[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
//     }

//     /// Borrow the data in the byte array with write access using the [Field] API.
//     /// See also [FieldSliceAccess::data_mut]
//     ///
//     /// # Example:
//     /// ```
//     /// use binary_layout::prelude::*;
//     ///
//     /// binary_layout!(my_layout, LittleEndian, {
//     ///     //... other fields ...
//     ///     some_field: [u8; 5],
//     ///     //... other fields
//     /// });
//     ///
//     /// fn func(storage_data: &mut [u8]) {
//     ///     let some_field: &mut [u8; 5] = my_layout::some_field::data_mut(storage_data);
//     /// }
//     /// ```
//     #[inline(always)]
//     fn data_mut(storage: &'a mut [u8]) -> &'a mut [u8; N] {
//         <&mut [u8; N]>::try_from(&mut storage[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
//     }
// }

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use crate::prelude::*;
    use crate::PrimitiveField;

    #[test]
    fn test_slice() {
        let mut storage = [0; 1024];

        type Field1 = PrimitiveField<[u8], LittleEndian, 5>;
        type Field2 = PrimitiveField<[u8], BigEndian, 7>;
        type Field3 = PrimitiveField<[u8], NativeEndian, 9>;

        Field1::data_mut(&mut storage)[..5].copy_from_slice(&[10, 20, 30, 40, 50]);
        Field2::data_mut(&mut storage)[..5].copy_from_slice(&[60, 70, 80, 90, 100]);
        Field3::data_mut(&mut storage)[..5].copy_from_slice(&[110, 120, 130, 140, 150]);

        assert_eq!(&[10, 20, 60, 70, 110], &Field1::data(&storage)[..5]);
        assert_eq!(&[60, 70, 110, 120, 130], &Field2::data(&storage)[..5]);
        assert_eq!(&[110, 120, 130, 140, 150], &Field3::data(&storage)[..5]);

        // Check types are correct
        let _a: &[u8] = Field1::data(&storage);
        let _b: &mut [u8] = Field1::data_mut(&mut storage);
    }

    #[test]
    fn test_array() {
        let mut storage = [0; 1024];

        type Field1 = PrimitiveField<[u8; 2], LittleEndian, 5>;
        type Field2 = PrimitiveField<[u8; 5], BigEndian, 6>;
        type Field3 = PrimitiveField<[u8; 5], NativeEndian, 7>;

        Field1::data_mut(&mut storage).copy_from_slice(&[10, 20]);
        Field2::data_mut(&mut storage).copy_from_slice(&[60, 70, 80, 90, 100]);
        Field3::data_mut(&mut storage).copy_from_slice(&[60, 70, 80, 90, 100]);

        assert_eq!(&[10, 60], Field1::data(&storage));
        assert_eq!(&[60, 60, 70, 80, 90], Field2::data(&storage));
        assert_eq!(&[60, 70, 80, 90, 100], Field3::data(&storage));

        assert_eq!(Some(2), PrimitiveField::<[u8; 2], LittleEndian, 5>::SIZE);
        assert_eq!(Some(5), PrimitiveField::<[u8; 5], BigEndian, 5>::SIZE);
        assert_eq!(Some(5), PrimitiveField::<[u8; 5], NativeEndian, 5>::SIZE);

        // Check types are correct
        let _a: &[u8; 2] = Field1::data(&storage);
        let _b: &mut [u8; 2] = Field1::data_mut(&mut storage);
    }
}
