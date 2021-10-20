use core::convert::TryFrom;
use core::marker::PhantomData;

use crate::endianness::{EndianKind, Endianness};

use super::{Field, FieldCopyAccess, FieldSliceAccess, SizedField};

/// A [PrimitiveField] is a [Field] that directly represents a primitive type like [u8], [i16], ...
/// See [Field] for more info on this API.
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

impl<T: ?Sized, E: Endianness, const OFFSET_: usize> Field for PrimitiveField<T, E, OFFSET_> {
    /// See [Field::Endian]
    type Endian = E;
    /// See [Field::OFFSET]
    const OFFSET: usize = OFFSET_;
}

macro_rules! int_field {
    ($type:ident) => {
        impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess for PrimitiveField<$type, E, OFFSET_> {
            /// See [FieldCopyAccess::HighLevelType]
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
                    let mut value = [0; core::mem::size_of::<$type>()];
                    value.copy_from_slice(
                        &storage.as_ref()[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())],
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
                    storage.as_mut()[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())]
                        .copy_from_slice(&value_as_bytes);
                }
            }
        }

        impl<E: Endianness, const OFFSET_: usize> SizedField for PrimitiveField<$type, E, OFFSET_> {
            /// See [SizedField::SIZE]
            const SIZE: usize = core::mem::size_of::<$type>();
        }
    };
}

int_field!(i8);
int_field!(i16);
int_field!(i32);
int_field!(i64);
int_field!(i128);
int_field!(u8);
int_field!(u16);
int_field!(u32);
int_field!(u64);
int_field!(u128);

macro_rules! float_field {
    ($type:ident) => {
        impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess for PrimitiveField<$type, E, OFFSET_> {
            /// See [FieldCopyAccess::HighLevelType]
            type HighLevelType = $type;

            doc_comment::doc_comment! {
                concat! {"
                Read the float field from a given data region, assuming the defined layout, using the [Field] API.

                # Example:

                ```
                use binary_layout::prelude::*;

                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_float_field: ", stringify!($type), "
                    //... other fields ...
                });

                fn func(storage_data: &[u8]) {
                    let read: ", stringify!($type), " = my_layout::some_float_field::read(storage_data);
                }
                ```

                # WARNING

                At it's core, this method uses [", stringify!($type), "::from_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.from_bits),
                which has some weird behavior around signaling and non-signaling `NaN` values.  Read the
                documentation for [", stringify!($type), "::from_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.from_bits) which
                explains the situation.
                "},
                fn read(storage: &[u8]) -> $type {
                    let mut value = [0; core::mem::size_of::<$type>()];
                    value.copy_from_slice(
                        &storage.as_ref()[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())],
                    );
                    match E::KIND {
                        EndianKind::Big => $type::from_be_bytes(value),
                        EndianKind::Little => $type::from_le_bytes(value),
                    }
                }
            }

            doc_comment::doc_comment! {
                concat! {"
                Write the float field to a given data region, assuming the defined layout, using the [Field] API.

                # Example:

                ```
                use binary_layout::prelude::*;

                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_float_field: ", stringify!($type), "
                    //... other fields ...
                });

                fn func(storage_data: &mut [u8]) {
                    my_layout::some_float_field::write(storage_data, 10.0);
                }
                ```

                # WARNING

                At it's core, this method uses [", stringify!($type), "::to_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.to_bits),
                which has some weird behavior around signaling and non-signaling `NaN` values.  Read the
                documentation for [", stringify!($type), "::to_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.to_bits) which
                explains the situation.
                "},
                fn write(storage: &mut [u8], value: $type) {
                    let value_as_bytes = match E::KIND {
                        EndianKind::Big => value.to_be_bytes(),
                        EndianKind::Little => value.to_le_bytes(),
                    };
                    storage.as_mut()[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())]
                        .copy_from_slice(&value_as_bytes);
                }
            }
        }

        impl<E: Endianness, const OFFSET_: usize> SizedField for PrimitiveField<$type, E, OFFSET_> {
            /// See [SizedField::SIZE]
            const SIZE: usize = core::mem::size_of::<$type>();
        }
    };
}

float_field!(f32);
float_field!(f64);

impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess
    for PrimitiveField<(), E, OFFSET_>
{
    /// See [FieldCopyAccess::HighLevelType]
    type HighLevelType = ();

    doc_comment::doc_comment! {
        concat! {"
                'Read' the `", stringify!(()), "`-typed field from a given data region, assuming the defined layout, using the [Field] API.

                # Example:

                ```
                use binary_layout::prelude::*;

                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_zst_field: ", stringify!(()), "
                    //... other fields ...
                });

                fn func(storage_data: &[u8]) {
                    let read: ", stringify!(()), " = my_layout::some_zst_field::read(storage_data);
                }
                ```

                In reality, this method doesn't do any work; `",
                stringify!(()), "` is a zero-sized type, so there's no work to
                do. This implementation exists solely to make writing derive
                macros simpler.
                "},
        #[allow(clippy::unused_unit)] // I don't want to remove this as it's part of the trait.
        fn read(_storage: &[u8]) -> () {
            ()
        }
    }

    doc_comment::doc_comment! {
        concat! {"
                'Write' the `", stringify!(()), "`-typed field to a given data region, assuming the defined layout, using the [Field] API.

                # Example:

                ```
                use binary_layout::prelude::*;

                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_zst_field: ", stringify!(()), "
                    //... other fields ...
                });

                fn func(storage_data: &mut [u8]) {
                    my_layout::some_zst_field::write(storage_data, ());
                }
                ```

                # WARNING

                In reality, this method doesn't do any work; `",
                stringify!(()), "` is a zero-sized type, so there's no work to
                do. This implementation exists solely to make writing derive
                macros simpler.
                "},
        #[allow(clippy::unused_unit)] // I don't want to remove this as it's part of the trait.
        fn write(_storage: &mut [u8], _value: ()) {
            ()
        }
    }
}

impl<E: Endianness, const OFFSET_: usize> SizedField
    for PrimitiveField<(), E, OFFSET_>
{
    /// See [SizedField::SIZE]
    const SIZE: usize = core::mem::size_of::<()>();
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
    use crate::endianness::{BigEndian, LittleEndian};
    use core::convert::TryInto;

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
    fn test_i128_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i128, LittleEndian, 5>;
        type Field2 = PrimitiveField<i128, LittleEndian, 200>;

        Field1::write(&mut storage, 10i128.pow(30));
        Field2::write(&mut storage, -(10i128.pow(28)));

        assert_eq!(
            10i128.pow(30),
            i128::from_le_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            -(10i128.pow(28)),
            i128::from_le_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(10i128.pow(30), Field1::read(&storage));
        assert_eq!(-(10i128.pow(28)), Field2::read(&storage));

        assert_eq!(16, PrimitiveField::<i128, LittleEndian, 5>::SIZE);
        assert_eq!(16, PrimitiveField::<i128, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i128_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i128, BigEndian, 5>;
        type Field2 = PrimitiveField<i128, BigEndian, 200>;

        Field1::write(&mut storage, 10i128.pow(30));
        Field2::write(&mut storage, -(10i128.pow(28)));

        assert_eq!(
            10i128.pow(30),
            i128::from_be_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            -(10i128.pow(28)),
            i128::from_be_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(10i128.pow(30), Field1::read(&storage));
        assert_eq!(-(10i128.pow(28)), Field2::read(&storage));

        assert_eq!(16, PrimitiveField::<i128, BigEndian, 5>::SIZE);
        assert_eq!(16, PrimitiveField::<i128, BigEndian, 5>::SIZE);
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
    fn test_u128_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u128, LittleEndian, 5>;
        type Field2 = PrimitiveField<u128, LittleEndian, 200>;

        Field1::write(&mut storage, 10u128.pow(30));
        Field2::write(&mut storage, 10u128.pow(28));

        assert_eq!(
            10u128.pow(30),
            u128::from_le_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            10u128.pow(28),
            u128::from_le_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(10u128.pow(30), Field1::read(&storage));
        assert_eq!(10u128.pow(28), Field2::read(&storage));

        assert_eq!(16, PrimitiveField::<u128, LittleEndian, 5>::SIZE);
        assert_eq!(16, PrimitiveField::<u128, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u128_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u128, BigEndian, 5>;
        type Field2 = PrimitiveField<u128, BigEndian, 200>;

        Field1::write(&mut storage, 10u128.pow(30));
        Field2::write(&mut storage, 10u128.pow(28));

        assert_eq!(
            10u128.pow(30),
            u128::from_be_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            10u128.pow(28),
            u128::from_be_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(10u128.pow(30), Field1::read(&storage));
        assert_eq!(10u128.pow(28), Field2::read(&storage));

        assert_eq!(16, PrimitiveField::<u128, BigEndian, 5>::SIZE);
        assert_eq!(16, PrimitiveField::<u128, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_f32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<f32, LittleEndian, 5>;
        type Field2 = PrimitiveField<f32, LittleEndian, 20>;

        Field1::write(&mut storage, 10f32.powf(8.31));
        Field2::write(&mut storage, -(10f32.powf(7.31)));

        assert_eq!(
            10f32.powf(8.31),
            f32::from_le_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            -(10f32.powf(7.31)),
            f32::from_le_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10f32.powf(8.31), Field1::read(&storage));
        assert_eq!(-(10f32.powf(7.31)), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<f32, LittleEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<f32, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_f32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<f32, BigEndian, 5>;
        type Field2 = PrimitiveField<f32, BigEndian, 20>;

        Field1::write(&mut storage, 10f32.powf(8.31));
        Field2::write(&mut storage, -(10f32.powf(7.31)));

        assert_eq!(
            10f32.powf(8.31),
            f32::from_be_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            -(10f32.powf(7.31)),
            f32::from_be_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10f32.powf(8.31), Field1::read(&storage));
        assert_eq!(-(10f32.powf(7.31)), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<f32, BigEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<f32, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_f64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<f64, LittleEndian, 5>;
        type Field2 = PrimitiveField<f64, LittleEndian, 20>;

        Field1::write(&mut storage, 10f64.powf(15.31));
        Field2::write(&mut storage, -(10f64.powf(15.31)));

        assert_eq!(
            10f64.powf(15.31),
            f64::from_le_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            -(10f64.powf(15.31)),
            f64::from_le_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10f64.powf(15.31), Field1::read(&storage));
        assert_eq!(-(10f64.powf(15.31)), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<f64, LittleEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<f64, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_f64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<f64, BigEndian, 5>;
        type Field2 = PrimitiveField<f64, BigEndian, 20>;

        Field1::write(&mut storage, 10f64.powf(15.31));
        Field2::write(&mut storage, -(10f64.powf(15.31)));

        assert_eq!(
            10f64.powf(15.31),
            f64::from_be_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            -(10f64.powf(15.31)),
            f64::from_be_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10f64.powf(15.31), Field1::read(&storage));
        assert_eq!(-(10f64.powf(15.31)), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<f64, BigEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<f64, BigEndian, 5>::SIZE);
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

    #[allow(clippy::unit_cmp)]
    #[test]
    fn test_unit_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<(), BigEndian, 5>;
        type Field2 = PrimitiveField<(), BigEndian, 20>;

        Field1::write(&mut storage, ());
        Field2::write(&mut storage, ());

        assert_eq!((), Field1::read(&storage));
        assert_eq!((), Field2::read(&storage));

        assert_eq!(0, PrimitiveField::<(), BigEndian, 5>::SIZE);
        assert_eq!(0, PrimitiveField::<(), BigEndian, 20>::SIZE);

        // Zero-sized types do not mutate the storage, so it should remain
        // unchanged for all of time.
        assert_eq!(storage, vec![0; 1024]);
    }

    #[allow(clippy::unit_cmp)]
    #[test]
    fn test_unit_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<(), LittleEndian, 5>;
        type Field2 = PrimitiveField<(), LittleEndian, 20>;

        Field1::write(&mut storage, ());
        Field2::write(&mut storage, ());

        assert_eq!((), Field1::read(&storage));
        assert_eq!((), Field2::read(&storage));

        assert_eq!(0, PrimitiveField::<(), LittleEndian, 5>::SIZE);
        assert_eq!(0, PrimitiveField::<(), LittleEndian, 20>::SIZE);

        // Zero-sized types do not mutate the storage, so it should remain
        // unchanged for all of time.
        assert_eq!(storage, vec![0; 1024]);
    }
}
