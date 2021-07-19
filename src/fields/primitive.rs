use std::convert::TryFrom;
use std::marker::PhantomData;

use crate::endianness::{EndianKind, Endianness};

use super::{IField, IFieldCopyAccess, IFieldSliceAccess, ISizedField};

/// A field represents one of the fields in the data layout and offers accessors
/// for it. It remembers the offset of the field in its const generic parameter
/// and the accessors use that to access the field.
///
/// A field does not hold any data storage, so if you use this API directly, you have to pass in
/// the storage pointer for each call. If you want an API object that remembers the storage,
/// take a look at the [FieldView] based API instead.
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
pub struct PrimitiveField<T: ?Sized, E: Endianness, const OFFSET_: usize> {
    _p1: PhantomData<T>,
    _p2: PhantomData<E>,
}

impl<T: ?Sized, E: Endianness, const OFFSET_: usize> IField for PrimitiveField<T, E, OFFSET_> {
    // TODO Doc
    type Endian = E;
    // TODO Doc
    const OFFSET: usize = OFFSET_;
}

macro_rules! int_field {
    ($type:ident) => {
        impl<E: Endianness, const OFFSET_: usize> IFieldCopyAccess for PrimitiveField<$type, E, OFFSET_> {
            // TODO Doc
            type HighLevelType = $type;

            doc_comment::doc_comment! {
                concat! {"
                Read the integer field from a given data region, assuming the defined layout, using the [Field] API.
                
                # Example:
                
                ```
                use binary_layout::prelude::*;
                    
                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_integer_field: ", stringify!($type), "
                    //... other fields ...
                });

                fn func(storage_data: &[u8]) {
                    let read: ", stringify!($type), " = my_layout::some_integer_field::read(storage_data);
                }
                ```
                "},
                fn read(storage: &[u8]) -> $type {
                    let mut value = [0; std::mem::size_of::<$type>()];
                    value.copy_from_slice(
                        &storage.as_ref()[Self::OFFSET..(Self::OFFSET + std::mem::size_of::<$type>())],
                    );
                    match E::KIND {
                        EndianKind::Big => $type::from_be_bytes(value),
                        EndianKind::Little => $type::from_le_bytes(value),
                    }
                }
            }

            doc_comment::doc_comment! {
                concat! {"
                Write the integer field to a given data region, assuming the defined layout, using the [Field] API.
                
                # Example:
                
                ```
                use binary_layout::prelude::*;
                    
                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_integer_field: ", stringify!($type), "
                    //... other fields ...
                });

                fn func(storage_data: &mut [u8]) {
                    my_layout::some_integer_field::write(storage_data, 10);
                }
                ```
                "},
                fn write(storage: &mut [u8], value: $type) {
                    let value_as_bytes = match E::KIND {
                        EndianKind::Big => value.to_be_bytes(),
                        EndianKind::Little => value.to_le_bytes(),
                    };
                    storage.as_mut()[Self::OFFSET..(Self::OFFSET + std::mem::size_of::<$type>())]
                        .copy_from_slice(&value_as_bytes);
                }
            }
        }

        impl<E: Endianness, const OFFSET_: usize> ISizedField for PrimitiveField<$type, E, OFFSET_> {
            // TODO Doc
            const SIZE: usize = std::mem::size_of::<$type>();
        }
        // impl FieldSize for $type {
        //     const SIZE: usize = std::mem::size_of::<$type>();
        // }
    };
}

int_field!(i8);
int_field!(i16);
int_field!(i32);
int_field!(i64);
int_field!(u8);
int_field!(u16);
int_field!(u32);
int_field!(u64);

/// Field type `[u8]`:
/// This field represents an [open ended byte array](crate#open-ended-byte-arrays-u8).
/// In this impl, we define accessors for such fields.
impl<'a, E: Endianness, const OFFSET_: usize> IFieldSliceAccess<'a>
    for PrimitiveField<[u8], E, OFFSET_>
{
    // TODO Docs
    type SliceType = &'a [u8];
    // TODO Docs
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
        &storage.as_ref()[Self::OFFSET..]
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
        &mut storage.as_mut()[Self::OFFSET..]
    }
}

/// Field type `[u8; N]`:
/// This field represents a [fixed size byte array](crate#fixed-size-byte-arrays-u8-n).
/// In this impl, we define accessors for such fields.
impl<'a, E: Endianness, const N: usize, const OFFSET_: usize> IFieldSliceAccess<'a>
    for PrimitiveField<[u8; N], E, OFFSET_>
{
    // TODO Docs
    type SliceType = &'a [u8; N];
    // TODO Docs
    type MutSliceType = &'a mut [u8; N];

    /// Borrow the data in the byte array with read access using the [Field] API.
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
        <&[u8; N]>::try_from(&storage.as_ref()[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
    }

    /// Borrow the data in the byte array with write access using the [Field] API.
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
        <&mut [u8; N]>::try_from(&mut storage.as_mut()[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
    }
}
impl<E: Endianness, const N: usize, const OFFSET_: usize> ISizedField
    for PrimitiveField<[u8; N], E, OFFSET_>
{
    const SIZE: usize = N;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endianness::{BigEndian, LittleEndian};
    use std::convert::TryInto;

    #[test]
    fn test_i8_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i8, LittleEndian, 5>;
        type Field2 = PrimitiveField<i8, LittleEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, -20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(-20, Field2::read(&storage));

        assert_eq!(50, i8::from_le_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            -20,
            i8::from_le_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, PrimitiveField::<i8, LittleEndian, 5>::SIZE);
        assert_eq!(1, PrimitiveField::<i8, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i8_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i8, BigEndian, 5>;
        type Field2 = PrimitiveField<i8, BigEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, -20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(-20, Field2::read(&storage));

        assert_eq!(50, i8::from_be_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            -20,
            i8::from_be_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, PrimitiveField::<i8, BigEndian, 5>::SIZE);
        assert_eq!(1, PrimitiveField::<i8, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_i16_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i16, LittleEndian, 5>;
        type Field2 = PrimitiveField<i16, LittleEndian, 20>;

        Field1::write(&mut storage, 500);
        Field2::write(&mut storage, -2000);

        assert_eq!(
            500,
            i16::from_le_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            -2000,
            i16::from_le_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(500, Field1::read(&storage));
        assert_eq!(-2000, Field2::read(&storage));

        assert_eq!(2, PrimitiveField::<i16, LittleEndian, 5>::SIZE);
        assert_eq!(2, PrimitiveField::<i16, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i16_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i16, BigEndian, 5>;
        type Field2 = PrimitiveField<i16, BigEndian, 20>;

        Field1::write(&mut storage, 500);
        Field2::write(&mut storage, -2000);

        assert_eq!(
            500,
            i16::from_be_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            -2000,
            i16::from_be_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(500, Field1::read(&storage));
        assert_eq!(-2000, Field2::read(&storage));

        assert_eq!(2, PrimitiveField::<i16, BigEndian, 5>::SIZE);
        assert_eq!(2, PrimitiveField::<i16, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_i32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i32, LittleEndian, 5>;
        type Field2 = PrimitiveField<i32, LittleEndian, 20>;

        Field1::write(&mut storage, 10i32.pow(8));
        Field2::write(&mut storage, -(10i32.pow(7)));

        assert_eq!(
            10i32.pow(8),
            i32::from_le_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            -(10i32.pow(7)),
            i32::from_le_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10i32.pow(8), Field1::read(&storage));
        assert_eq!(-(10i32.pow(7)), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<i32, LittleEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<i32, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i32, BigEndian, 5>;
        type Field2 = PrimitiveField<i32, BigEndian, 20>;

        Field1::write(&mut storage, 10i32.pow(8));
        Field2::write(&mut storage, -(10i32.pow(7)));

        assert_eq!(
            10i32.pow(8),
            i32::from_be_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            -(10i32.pow(7)),
            i32::from_be_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10i32.pow(8), Field1::read(&storage));
        assert_eq!(-(10i32.pow(7)), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<i32, BigEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<i32, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_i64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i64, LittleEndian, 5>;
        type Field2 = PrimitiveField<i64, LittleEndian, 20>;

        Field1::write(&mut storage, 10i64.pow(15));
        Field2::write(&mut storage, -(10i64.pow(14)));

        assert_eq!(
            10i64.pow(15),
            i64::from_le_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            -(10i64.pow(14)),
            i64::from_le_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10i64.pow(15), Field1::read(&storage));
        assert_eq!(-(10i64.pow(14)), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<i64, LittleEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<i64, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i64, BigEndian, 5>;
        type Field2 = PrimitiveField<i64, BigEndian, 20>;

        Field1::write(&mut storage, 10i64.pow(15));
        Field2::write(&mut storage, -(10i64.pow(14)));

        assert_eq!(
            10i64.pow(15),
            i64::from_be_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            -(10i64.pow(14)),
            i64::from_be_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10i64.pow(15), Field1::read(&storage));
        assert_eq!(-(10i64.pow(14)), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<i64, BigEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<i64, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u8_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u8, LittleEndian, 5>;
        type Field2 = PrimitiveField<u8, LittleEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, 20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(20, Field2::read(&storage));

        assert_eq!(50, u8::from_le_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            20,
            u8::from_le_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, PrimitiveField::<u8, LittleEndian, 5>::SIZE);
        assert_eq!(1, PrimitiveField::<u8, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u8_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u8, BigEndian, 5>;
        type Field2 = PrimitiveField<u8, BigEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, 20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(20, Field2::read(&storage));

        assert_eq!(50, u8::from_be_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            20,
            u8::from_be_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, PrimitiveField::<u8, BigEndian, 5>::SIZE);
        assert_eq!(1, PrimitiveField::<u8, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u16_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u16, LittleEndian, 5>;
        type Field2 = PrimitiveField<u16, LittleEndian, 20>;

        Field1::write(&mut storage, 500);
        Field2::write(&mut storage, 2000);

        assert_eq!(
            500,
            u16::from_le_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            2000,
            u16::from_le_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(500, Field1::read(&storage));
        assert_eq!(2000, Field2::read(&storage));

        assert_eq!(2, PrimitiveField::<u16, LittleEndian, 5>::SIZE);
        assert_eq!(2, PrimitiveField::<u16, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u16_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u16, BigEndian, 5>;
        type Field2 = PrimitiveField<u16, BigEndian, 20>;

        Field1::write(&mut storage, 500);
        Field2::write(&mut storage, 2000);

        assert_eq!(
            500,
            u16::from_be_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            2000,
            u16::from_be_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(500, Field1::read(&storage));
        assert_eq!(2000, Field2::read(&storage));

        assert_eq!(2, PrimitiveField::<u16, BigEndian, 5>::SIZE);
        assert_eq!(2, PrimitiveField::<u16, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u32, LittleEndian, 5>;
        type Field2 = PrimitiveField<u32, LittleEndian, 20>;

        Field1::write(&mut storage, 10u32.pow(8));
        Field2::write(&mut storage, 10u32.pow(7));

        assert_eq!(
            10u32.pow(8),
            u32::from_le_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10u32.pow(7),
            u32::from_le_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10u32.pow(8), Field1::read(&storage));
        assert_eq!(10u32.pow(7), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<u32, LittleEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<u32, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u32, BigEndian, 5>;
        type Field2 = PrimitiveField<u32, BigEndian, 20>;

        Field1::write(&mut storage, 10u32.pow(8));
        Field2::write(&mut storage, 10u32.pow(7));

        assert_eq!(
            10u32.pow(8),
            u32::from_be_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10u32.pow(7),
            u32::from_be_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10u32.pow(8), Field1::read(&storage));
        assert_eq!(10u32.pow(7), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<u32, BigEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<u32, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u64, LittleEndian, 5>;
        type Field2 = PrimitiveField<u64, LittleEndian, 20>;

        Field1::write(&mut storage, 10u64.pow(15));
        Field2::write(&mut storage, 10u64.pow(14));

        assert_eq!(
            10u64.pow(15),
            u64::from_le_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10u64.pow(14),
            u64::from_le_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10u64.pow(15), Field1::read(&storage));
        assert_eq!(10u64.pow(14), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<u64, LittleEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<u64, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u64, BigEndian, 5>;
        type Field2 = PrimitiveField<u64, BigEndian, 20>;

        Field1::write(&mut storage, 10u64.pow(15));
        Field2::write(&mut storage, 10u64.pow(14));

        assert_eq!(
            10u64.pow(15),
            u64::from_be_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10u64.pow(14),
            u64::from_be_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10u64.pow(15), Field1::read(&storage));
        assert_eq!(10u64.pow(14), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<u64, BigEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<u64, BigEndian, 5>::SIZE);
    }

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
