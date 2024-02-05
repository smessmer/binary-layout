use core::convert::Infallible;

use super::{FieldCopyAccess, PrimitiveField};
use crate::endianness::{EndianKind, Endianness};
use crate::fields::primitive::view::FieldView;
use crate::fields::{Field, StorageIntoFieldView, StorageToFieldView};

macro_rules! nonzero_int_field {
    ($type:ty, $zero_type:ty) => {
        impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess for PrimitiveField<$type, E, OFFSET_> {
            /// See [FieldCopyAccess::ReadError]
            type ReadError = NonZeroIsZeroError;
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

                fn func(storage_data: &[u8]) -> Result<",stringify!($type), ", NonZeroIsZeroError>{
                    let read: ", stringify!($type), " = my_layout::some_integer_field::try_read(storage_data)?;
                    Ok(read)
                }
                ```
                "},
                #[inline(always)]
                fn try_read(storage: &[u8]) -> Result<$type, NonZeroIsZeroError> {
                    // TODO Don't initialize memory
                    let mut value = [0; core::mem::size_of::<$type>()];
                    value.copy_from_slice(
                        &storage[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())],
                    );
                    let value = match E::KIND {
                        EndianKind::Big => <$zero_type>::from_be_bytes(value),
                        EndianKind::Little => <$zero_type>::from_le_bytes(value),
                        EndianKind::Native => <$zero_type>::from_ne_bytes(value)
                    };
                    <$type>::new(value).ok_or(NonZeroIsZeroError(()))
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
                    let value = ", stringify!($type), "::new(10).unwrap();
                    my_layout::some_integer_field::try_write(storage_data, value).unwrap();
                }
                ```
                "},
                #[inline(always)]
                fn try_write(storage: &mut [u8], value: $type) -> Result<(), Infallible> {
                    let value_as_bytes = match E::KIND {
                        EndianKind::Big => value.get().to_be_bytes(),
                        EndianKind::Little => value.get().to_le_bytes(),
                        EndianKind::Native => value.get().to_ne_bytes(),
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

/// This error is thrown when trying to read a non-zero integer type, e.g. [NonZeroU32],
/// but the data being read was actually zero.
#[derive(Debug)]
pub struct NonZeroIsZeroError(pub(crate) ());

impl core::fmt::Display for NonZeroIsZeroError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "NonZeroIsZeroError")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NonZeroIsZeroError {}

nonzero_int_field!(core::num::NonZeroI8, i8);
nonzero_int_field!(core::num::NonZeroI16, i16);
nonzero_int_field!(core::num::NonZeroI32, i32);
nonzero_int_field!(core::num::NonZeroI64, i64);
nonzero_int_field!(core::num::NonZeroI128, i128);
nonzero_int_field!(core::num::NonZeroU8, u8);
nonzero_int_field!(core::num::NonZeroU16, u16);
nonzero_int_field!(core::num::NonZeroU32, u32);
nonzero_int_field!(core::num::NonZeroU64, u64);
nonzero_int_field!(core::num::NonZeroU128, u128);

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use crate::prelude::*;
    use crate::PrimitiveField;

    macro_rules! test_nonzero {
        ($type:ty, $underlying_type:ty, $expected_size:expr, $value1:expr, $value2:expr) => {
            test_nonzero!(@case, $type, $underlying_type, $expected_size, $value1, $value2, little, LittleEndian, from_le_bytes);
            test_nonzero!(@case, $type, $underlying_type, $expected_size, $value1, $value2, big, BigEndian, from_be_bytes);
            test_nonzero!(@case, $type, $underlying_type, $expected_size, $value1, $value2, native, NativeEndian, from_ne_bytes);
        };
        (@case, $type:ty, $underlying_type:ty, $expected_size:expr, $value1:expr, $value2: expr, $endian:ident, $endian_type:ty, $endian_fn:ident) => {
            $crate::internal::paste! {
                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_tryread_write>]() {
                    let mut storage = vec![0; 1024];

                    let value1 = <$type>::new($value1).unwrap();
                    let value2 = <$type>::new($value2).unwrap();

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;

                    Field1::write(&mut storage, value1);
                    Field2::write(&mut storage, value2);

                    // TODO Test reading a zero
                    assert_eq!(value1, Field1::try_read(&storage).unwrap());
                    assert_eq!(value2, Field2::try_read(&storage).unwrap());

                    assert_eq!(value1, $type::new($underlying_type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(value2, $type::new($underlying_type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap())).unwrap());

                    assert_eq!(Some($expected_size), Field1::SIZE);
                    assert_eq!(5, Field1::OFFSET);
                    assert_eq!(Some($expected_size), Field2::SIZE);
                    assert_eq!(123, Field2::OFFSET);
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_tryread_trywrite>]() {
                    use crate::InfallibleResultExt;

                    let mut storage = vec![0; 1024];

                    let value1 = <$type>::new($value1).unwrap();
                    let value2 = <$type>::new($value2).unwrap();

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;

                    Field1::try_write(&mut storage, value1).infallible_unwrap();
                    Field2::try_write(&mut storage, value2).infallible_unwrap();

                    // TODO Test reading a zero
                    assert_eq!(value1, Field1::try_read(&storage).unwrap());
                    assert_eq!(value2, Field2::try_read(&storage).unwrap());

                    assert_eq!(value1, $type::new($underlying_type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(value2, $type::new($underlying_type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap())).unwrap());

                    assert_eq!(Some($expected_size), Field1::SIZE);
                    assert_eq!(5, Field1::OFFSET);
                    assert_eq!(Some($expected_size), Field2::SIZE);
                    assert_eq!(123, Field2::OFFSET);
                }
            }
        };
    }

    use core::num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    };

    test_nonzero!(NonZeroI8, i8, 1, 50, -20);
    test_nonzero!(NonZeroI16, i16, 2, 500, -2000);
    test_nonzero!(NonZeroI32, i32, 4, 10i32.pow(8), -(10i32.pow(7)));
    test_nonzero!(NonZeroI64, i64, 8, 10i64.pow(15), -(10i64.pow(14)));
    test_nonzero!(NonZeroI128, i128, 16, 10i128.pow(30), -(10i128.pow(28)));

    test_nonzero!(NonZeroU8, u8, 1, 50, 20);
    test_nonzero!(NonZeroU16, u16, 2, 500, 2000);
    test_nonzero!(NonZeroU32, u32, 4, 10u32.pow(8), (10u32.pow(7)));
    test_nonzero!(NonZeroU64, u64, 8, 10u64.pow(15), (10u64.pow(14)));
    test_nonzero!(NonZeroU128, u128, 16, 10u128.pow(30), (10u128.pow(28)));
}
