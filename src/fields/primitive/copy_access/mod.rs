use super::super::Field;
use super::PrimitiveField;

// TODO add support for the `char` primitive data type

/// This trait is implemented for fields with "try copy access",
/// i.e. fields that read/write data by copying it from/to the
/// binary blob, but where reading or writing can fail.
/// Examples of this are primitive types like NonZeroU8, NonZeroI32, ...
pub trait FieldCopyAccess: Field {
    /// Error type that can be thrown from [FieldCopyAccess::try_read]. You can set this to [Infallible](core::convert::Infallible) if the function does not throw an error.
    type ReadError;

    /// Error type that can be thrown from [FieldCopyAccess::try_write]. You can set this to [Infallible](core::convert::Infallible) if the function does not throw an error.
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
    /// use core::num::NonZeroU16;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   //... other fields ...
    ///   some_integer_field: core::num::NonZeroU16,
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
    /// use core::num::NonZeroU16;
    /// use core::convert::Infallible;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   //... other fields ...
    ///   some_integer_field: core::num::NonZeroU16,
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

mod primitive_float;
mod primitive_int;
mod primitive_nonzero_int;
mod primitive_unit;
mod read_write_ext;

pub use primitive_nonzero_int::NonZeroIsZeroError;
pub use read_write_ext::{FieldReadExt, FieldWriteExt};
