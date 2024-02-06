use core::convert::Infallible;

use super::{FieldCopyAccess, PrimitiveField};
use crate::endianness::{EndianKind, Endianness};
use crate::fields::primitive::view::FieldView;
use crate::fields::{Field, StorageIntoFieldView, StorageToFieldView};

macro_rules! int_field {
    ($type:ty) => {
        impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess for PrimitiveField<$type, E, OFFSET_> {
            /// See [FieldCopyAccess::ReadError]
            type ReadError = Infallible;
            /// See [FieldCopyAccess::WriteError]
            type WriteError = Infallible;
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

                fn func(storage_data: &[u8]) -> ",stringify!($type), " {
                    let read: ", stringify!($type), " = my_layout::some_integer_field::try_read(storage_data).unwrap();
                    read
                }
                ```
                "},
                #[inline(always)]
                fn try_read(storage: &[u8]) -> Result<$type, Infallible> {
                    let value: [u8; core::mem::size_of::<$type>()] = storage[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())].try_into().unwrap();
                    let value = match E::KIND {
                        EndianKind::Big => <$type>::from_be_bytes(value),
                        EndianKind::Little => <$type>::from_le_bytes(value),
                        EndianKind::Native => <$type>::from_ne_bytes(value)
                    };
                    Ok(value)
                }
            }

            doc_comment::doc_comment! {
                concat! {"
                Write the integer field to a given data region, assuming the defined layout, using the [Field] API.

                # Example:

                ```
                use binary_layout::prelude::*;
                use core::convert::Infallible;

                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_integer_field: ", stringify!($type), "
                    //... other fields ...
                });

                fn func(storage_data: &mut [u8]) {
                    my_layout::some_integer_field::try_write(storage_data, 10).unwrap();
                }
                ```
                "},
                #[inline(always)]
                fn try_write(storage: &mut [u8], value: $type) -> Result<(), Infallible> {
                    let value_as_bytes = match E::KIND {
                        EndianKind::Big => value.to_be_bytes(),
                        EndianKind::Little => value.to_le_bytes(),
                        EndianKind::Native => value.to_ne_bytes(),
                    };
                    storage[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())]
                        .copy_from_slice(&value_as_bytes);
                    Ok(())
                }
            }
        }

        impl_field_traits!($type);
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::PrimitiveField;

    macro_rules! test_int {
        ($type:ty, $expected_size:expr, $value1:expr, $value2:expr) => {
            test_int!(@case, $type, $expected_size, $value1, $value2, little, LittleEndian, from_le_bytes);
            test_int!(@case, $type, $expected_size, $value1, $value2, big, BigEndian, from_be_bytes);
            test_int!(@case, $type, $expected_size, $value1, $value2, native, NativeEndian, from_ne_bytes);
        };
        (@case, $type:ty, $expected_size:expr, $value1:expr, $value2: expr, $endian:ident, $endian_type:ty, $endian_fn:ident) => {
            $crate::internal::paste! {
                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_read_write>]() {
                    let mut storage = [0; 1024];

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;

                    Field1::write(&mut storage, $value1);
                    Field2::write(&mut storage, $value2);

                    // TODO Test reading a zero
                    assert_eq!($value1, Field1::read(&storage));
                    assert_eq!($value2, Field2::read(&storage));

                    assert_eq!($value1, $type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap()));

                    assert_eq!(Some($expected_size), Field1::SIZE);
                    assert_eq!(5, Field1::OFFSET);
                    assert_eq!(Some($expected_size), Field2::SIZE);
                    assert_eq!(123, Field2::OFFSET);
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_tryread_trywrite>]() {
                    use crate::InfallibleResultExt;

                    let mut storage = [0; 1024];

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;

                    Field1::try_write(&mut storage, $value1).infallible_unwrap();
                    Field2::try_write(&mut storage, $value2).infallible_unwrap();

                    // TODO Test reading a zero
                    assert_eq!($value1, Field1::try_read(&storage).infallible_unwrap());
                    assert_eq!($value2, Field2::try_read(&storage).infallible_unwrap());

                    assert_eq!($value1, $type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap()));

                    assert_eq!(Some($expected_size), Field1::SIZE);
                    assert_eq!(5, Field1::OFFSET);
                    assert_eq!(Some($expected_size), Field2::SIZE);
                    assert_eq!(123, Field2::OFFSET);
                }
            }
        };
    }

    test_int!(i8, 1, 50, -20);
    test_int!(i16, 2, 500, -2000);
    test_int!(i32, 4, 10i32.pow(8), -(10i32.pow(7)));
    test_int!(i64, 8, 10i64.pow(15), -(10i64.pow(14)));
    test_int!(i128, 16, 10i128.pow(30), -(10i128.pow(28)));

    test_int!(u8, 1, 50, 20);
    test_int!(u16, 2, 500, 2000);
    test_int!(u32, 4, 10u32.pow(8), (10u32.pow(7)));
    test_int!(u64, 8, 10u64.pow(15), (10u64.pow(14)));
    test_int!(u128, 16, 10u128.pow(30), (10u128.pow(28)));
}
