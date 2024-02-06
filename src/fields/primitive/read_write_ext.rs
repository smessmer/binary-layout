use super::super::Field;
use super::copy_access::FieldCopyAccess;
use crate::utils::infallible::{InfallibleResultExt, IsInfallible};

/// This extension trait adds a [FieldReadExt::read] method to any type
/// supporting [FieldCopyAccess::try_read] that has an implementation
/// that cannot throw errors. This is a convenience function so that callers
/// can just call [FieldReadExt::read] instead of having to call [FieldCopyAccess::try_read]
/// and then calling [Result::unwrap] on the returned value.
pub trait FieldReadExt: Field {
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
}

/// This extension trait adds a [FieldWriteExt::write] method to any type
/// supporting [FieldCopyAccess::try_write] that has an implementation
/// that cannot throw errors. This is a convenience function so that callers
/// can just call [FieldWriteExt::write] instead of having to call [FieldCopyAccess::try_write]
/// and then calling [Result::unwrap] on the returned value.
pub trait FieldWriteExt: Field {
    /// The data type that is returned from read calls and has to be
    /// passed in to write calls. This can be different from the primitive
    /// type used in the binary blob, since that primitive type can be
    /// wrapped (see [WrappedField](crate::WrappedField) ) into a high level type before being returned from read
    /// calls (or vice versa unwrapped when writing).
    type HighLevelType;

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

impl<F> FieldReadExt for F
where
    F: FieldCopyAccess,
    F::ReadError: IsInfallible,
{
    type HighLevelType = F::HighLevelType;

    /// This implements a convenience method for reading any data type whose [FieldCopyAccess::try_read]
    /// does not throw errors.
    /// See [FieldCopyAccess::try_read].
    #[inline(always)]
    fn read(storage: &[u8]) -> Self::HighLevelType {
        F::try_read(storage).infallible_unwrap()
    }
}

// TODO Add tests for types that can only fail when reading, only fail when writing, fail when doing either.
//      For both, view API and field API.

impl<F> FieldWriteExt for F
where
    F: FieldCopyAccess,
    F::WriteError: IsInfallible,
{
    type HighLevelType = F::HighLevelType;

    /// This implements a convenience method for writing any data type whose [FieldCopyAccess::try_write]
    /// does not throw errors.
    /// See [FieldCopyAccess::try_write].
    #[inline(always)]
    fn write(storage: &mut [u8], value: Self::HighLevelType) {
        F::try_write(storage, value).infallible_unwrap()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use crate::prelude::*;
    use crate::PrimitiveField;

    macro_rules! test_primitive_copy_access {
        ($type:ty, $expected_size:expr, $value1:expr, $value2:expr) => {
            test_primitive_copy_access!(@case, $type, $expected_size, $value1, $value2, little, LittleEndian, from_le_bytes);
            test_primitive_copy_access!(@case, $type, $expected_size, $value1, $value2, big, BigEndian, from_be_bytes);
            test_primitive_copy_access!(@case, $type, $expected_size, $value1, $value2, native, NativeEndian, from_ne_bytes);
        };
        (@case, $type:ty, $expected_size:expr, $value1:expr, $value2: expr, $endian:ident, $endian_type:ty, $endian_fn:ident) => {
            $crate::internal::paste! {
                #[test]
                fn [<test_ $type _ $endian endian>]() {
                    let mut storage = [0; 1024];

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;

                    Field1::write(&mut storage, $value1);
                    Field2::write(&mut storage, $value2);

                    assert_eq!($value1, Field1::read(&storage));
                    assert_eq!($value2, Field2::read(&storage));

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

    test_primitive_copy_access!(i8, 1, 50, -20);
    test_primitive_copy_access!(i16, 2, 500, -2000);
    test_primitive_copy_access!(i32, 4, 10i32.pow(8), -(10i32.pow(7)));
    test_primitive_copy_access!(i64, 8, 10i64.pow(15), -(10i64.pow(14)));
    test_primitive_copy_access!(i128, 16, 10i128.pow(30), -(10i128.pow(28)));

    test_primitive_copy_access!(u8, 1, 50, 20);
    test_primitive_copy_access!(u16, 2, 500, 2000);
    test_primitive_copy_access!(u32, 4, 10u32.pow(8), (10u32.pow(7)));
    test_primitive_copy_access!(u64, 8, 10u64.pow(15), (10u64.pow(14)));
    test_primitive_copy_access!(u128, 16, 10u128.pow(30), (10u128.pow(28)));

    test_primitive_copy_access!(f32, 4, 10f32.powf(8.31), -(10f32.powf(7.31)));
    test_primitive_copy_access!(f64, 8, 10f64.powf(15.31), -(10f64.powf(15.31)));

    macro_rules! test_unit_copy_access {
        ($endian:ident, $endian_type:ty) => {
            $crate::internal::paste! {
                #[allow(clippy::unit_cmp)]
                #[test]
                fn [<test_unit_ $endian endian>]() {
                    let mut storage = [0; 1024];

                    type Field1 = PrimitiveField<(), $endian_type, 5>;
                    type Field2 = PrimitiveField<(), $endian_type, 123>;

                    Field1::write(&mut storage, ());
                    Field2::write(&mut storage, ());

                    assert_eq!((), Field1::read(&storage));
                    assert_eq!((), Field2::read(&storage));

                    assert_eq!(Some(0), Field1::SIZE);
                    assert_eq!(5, Field1::OFFSET);
                    assert_eq!(Some(0), Field2::SIZE);
                    assert_eq!(123, Field2::OFFSET);

                    // Zero-sized types do not mutate the storage, so it should remain
                    // unchanged for all of time.
                    assert_eq!(storage, [0; 1024]);
                }
            }
        };
    }

    test_unit_copy_access!(little, LittleEndian);
    test_unit_copy_access!(big, BigEndian);
    test_unit_copy_access!(native, NativeEndian);
}
