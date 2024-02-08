use crate::fields::{primitive::copy_access::FieldCopyAccess, Field};
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
    /// binary_layout!(my_layout, LittleEndian, {
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
    /// binary_layout!(my_layout, LittleEndian, {
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
