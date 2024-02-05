use super::super::{Field, StorageIntoFieldView, StorageToFieldView};
use super::view::FieldView;
use super::PrimitiveField;
use crate::endianness::{EndianKind, Endianness};
use std::convert::Infallible;

// TODO Should we implement FieldTryCopyAccess instead of FieldCopyAccess for all zeroable integer types, using try_read/try_write,
//      and then have a generic extension trait that adds read/write functions if try_read/try_write return Result<_, Infallible>?
//      Maybe rename FieldTryCopyAccess to FieldCopyAccess then, and add two separate extension traits, one for read and one for write.
//      This could then also be used to unify LayoutAs/TryLayoutAs. Any custom wrapper type that returns infallible could automatically
//      get read/write methods.
// TODO It's weird to have write for NonZero types return a Result if it can't fail. Can we do a type instead that only returns Result from read?

/// This trait is implemented for fields with "try copy access",
/// i.e. fields that read/write data by copying it from/to the
/// binary blob, but where reading or writing can fail.
/// Examples of this are primitive types like NonZeroU8, NonZeroI32, ...
pub trait FieldTryCopyAccess: Field {
    /// Error type that can be thrown from [FieldTryCopyAccess::read]. You can set this to [Infallible] if the function does not throw an error.
    type ReadError;

    /// Error type that can be thrown from [FieldTryCopyAccess::write]. You can set this to [Infallible] if the function does not throw an error.
    type WriteError;

    /// The data type that is returned from read calls and has to be
    /// passed in to write calls. This can be different from the primitive
    /// type used in the binary blob, since that primitive type can be
    /// wrapped (see [WrappedField](crate::WrappedField) ) into a high level type before being returned from read
    /// calls (or vice versa unwrapped when writing).
    type HighLevelType;

    /// Read the field from a given data region, assuming the defined layout, using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    /// use std::num::NonZeroU16;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   //... other fields ...
    ///   some_integer_field: std::num::NonZeroU16,
    ///   //... other fields ...
    /// });
    ///
    /// fn func(storage_data: &[u8]) -> Result<NonZeroU16, NonZeroIsZeroError> {
    ///   let read: NonZeroU16 = my_layout::some_integer_field::try_read(storage_data)?;
    ///   Ok(read)
    /// }
    /// ```
    fn try_read(storage: &[u8]) -> Result<Self::HighLevelType, Self::ReadError>;

    /// Write the field to a given data region, assuming the defined layout, using the [Field] API.
    ///
    /// # Example:
    ///
    /// ```
    /// use binary_layout::prelude::*;
    /// use std::num::NonZeroU16;
    /// use std::convert::Infallible;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   //... other fields ...
    ///   some_integer_field: std::num::NonZeroU16,
    ///   //... other fields ...
    /// });
    ///
    /// fn func(storage_data: &mut [u8]) -> Result<(), Infallible> {
    ///   let value = NonZeroU16::new(10).unwrap();
    ///   my_layout::some_integer_field::try_write(storage_data, value)?;
    ///   Ok(())
    /// }
    /// ```
    fn try_write(storage: &mut [u8], v: Self::HighLevelType) -> Result<(), Self::WriteError>;
}

macro_rules! impl_field_traits {
    ($type: ty) => {
        impl<E: Endianness, const OFFSET_: usize> Field for PrimitiveField<$type, E, OFFSET_> {
            /// See [Field::Endian]
            type Endian = E;
            /// See [Field::OFFSET]
            const OFFSET: usize = OFFSET_;
            /// See [Field::SIZE]
            const SIZE: Option<usize> = Some(core::mem::size_of::<$type>());
        }

        impl<'a, E: Endianness, const OFFSET_: usize> StorageToFieldView<&'a [u8]>
            for PrimitiveField<$type, E, OFFSET_>
        {
            type View = FieldView<&'a [u8], Self>;

            #[inline(always)]
            fn view(storage: &'a [u8]) -> Self::View {
                Self::View::new(storage)
            }
        }

        impl<'a, E: Endianness, const OFFSET_: usize> StorageToFieldView<&'a mut [u8]>
            for PrimitiveField<$type, E, OFFSET_>
        {
            type View = FieldView<&'a mut [u8], Self>;

            #[inline(always)]
            fn view(storage: &'a mut [u8]) -> Self::View {
                Self::View::new(storage)
            }
        }

        impl<S: AsRef<[u8]>, E: Endianness, const OFFSET_: usize> StorageIntoFieldView<S>
            for PrimitiveField<$type, E, OFFSET_>
        {
            type View = FieldView<S, Self>;

            #[inline(always)]
            fn into_view(storage: S) -> Self::View {
                Self::View::new(storage)
            }
        }
    };
}

macro_rules! nonzero_int_field {
    ($type:ty, $zero_type:ty) => {
        impl<E: Endianness, const OFFSET_: usize> FieldTryCopyAccess for PrimitiveField<$type, E, OFFSET_> {
            /// See [FieldTryCopyAccess::ReadError]
            type ReadError = NonZeroIsZeroError;
            /// See [FieldTryCopyAccess::WriteError]
            type WriteError = Infallible;
            /// See [FieldTryCopyAccess::HighLevelType]
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
                use std::convert::Infallible;

                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_integer_field: ", stringify!($type), "
                    //... other fields ...
                });

                fn func(storage_data: &mut [u8]) -> Result<(), Infallible> {
                    let value = ", stringify!($type), "::new(10).unwrap();
                    my_layout::some_integer_field::try_write(storage_data, value)?;
                    Ok(())
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

impl std::fmt::Display for NonZeroIsZeroError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "NonZeroIsZeroError")
    }
}

impl std::error::Error for NonZeroIsZeroError {}

nonzero_int_field!(std::num::NonZeroI8, i8);
nonzero_int_field!(std::num::NonZeroI16, i16);
nonzero_int_field!(std::num::NonZeroI32, i32);
nonzero_int_field!(std::num::NonZeroI64, i64);
nonzero_int_field!(std::num::NonZeroI128, i128);
nonzero_int_field!(std::num::NonZeroU8, u8);
nonzero_int_field!(std::num::NonZeroU16, u16);
nonzero_int_field!(std::num::NonZeroU32, u32);
nonzero_int_field!(std::num::NonZeroU64, u64);
nonzero_int_field!(std::num::NonZeroU128, u128);

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use crate::prelude::*;
    use crate::PrimitiveField;

    macro_rules! test_nonzero_try_copy_access {
        ($type:ty, $underlying_type:ty, $expected_size:expr, $value1:expr, $value2:expr) => {
            test_nonzero_try_copy_access!(@case, $type, $underlying_type, $expected_size, $value1, $value2, little, LittleEndian, from_le_bytes);
            test_nonzero_try_copy_access!(@case, $type, $underlying_type, $expected_size, $value1, $value2, big, BigEndian, from_be_bytes);
            test_nonzero_try_copy_access!(@case, $type, $underlying_type, $expected_size, $value1, $value2, native, NativeEndian, from_ne_bytes);
        };
        (@case, $type:ty, $underlying_type:ty, $expected_size:expr, $value1:expr, $value2: expr, $endian:ident, $endian_type:ty, $endian_fn:ident) => {
            $crate::internal::paste! {
                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian>]() {
                    let mut storage = vec![0; 1024];

                    let value1 = <$type>::new($value1).unwrap();
                    let value2 = <$type>::new($value2).unwrap();

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;

                    // TODO Use infallible_unwrap here
                    Field1::try_write(&mut storage, value1).unwrap();
                    Field2::try_write(&mut storage, value2).unwrap();

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

    use std::num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    };

    test_nonzero_try_copy_access!(NonZeroI8, i8, 1, 50, -20);
    test_nonzero_try_copy_access!(NonZeroI16, i16, 2, 500, -2000);
    test_nonzero_try_copy_access!(NonZeroI32, i32, 4, 10i32.pow(8), -(10i32.pow(7)));
    test_nonzero_try_copy_access!(NonZeroI64, i64, 8, 10i64.pow(15), -(10i64.pow(14)));
    test_nonzero_try_copy_access!(NonZeroI128, i128, 16, 10i128.pow(30), -(10i128.pow(28)));

    test_nonzero_try_copy_access!(NonZeroU8, u8, 1, 50, 20);
    test_nonzero_try_copy_access!(NonZeroU16, u16, 2, 500, 2000);
    test_nonzero_try_copy_access!(NonZeroU32, u32, 4, 10u32.pow(8), (10u32.pow(7)));
    test_nonzero_try_copy_access!(NonZeroU64, u64, 8, 10u64.pow(15), (10u64.pow(14)));
    test_nonzero_try_copy_access!(NonZeroU128, u128, 16, 10u128.pow(30), (10u128.pow(28)));
}
