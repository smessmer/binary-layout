use super::PrimitiveField;
use crate::data_types::{
    primitive::{PrimitiveRead, PrimitiveWrite},
    DataTypeMetadata,
};
use crate::endianness::Endianness;
use crate::fields::Field;
use crate::view::PrimitiveFieldView;

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
    /// binary_layout!(my_layout, LittleEndian, {
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
    /// binary_layout!(my_layout, LittleEndian, {
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

impl<T: DataTypeMetadata + PrimitiveRead + PrimitiveWrite, E: Endianness, const OFFSET_: usize>
    FieldCopyAccess for PrimitiveField<T, E, OFFSET_>
{
    /// See [FieldCopyAccess::ReadError]
    type ReadError = <T as PrimitiveRead>::Error;
    /// See [FieldCopyAccess::WriteError]
    type WriteError = <T as PrimitiveWrite>::Error;
    /// See [FieldCopyAccess::HighLevelType]
    type HighLevelType = T;

    doc_comment::doc_comment! {
        concat! {"
        Read the integer field from a given data region, assuming the defined layout, using the [Field] API.

        # Example:

        ```
        use binary_layout::prelude::*;

        binary_layout!(my_layout, LittleEndian, {
            //... other fields ...
            some_integer_field: u32,
            //... other fields ...
        });

        fn func(storage_data: &[u8]) -> u32 {
            let read: u32 = my_layout::some_integer_field::try_read(storage_data).unwrap();
            read
        }
        ```
        "},
        #[inline(always)]
        fn try_read(storage: &[u8]) -> Result<T, Self::ReadError> {
            let storage = if let Some(size) = Self::SIZE {
                &storage[Self::OFFSET..(Self::OFFSET + size)]
            } else {
                &storage[Self::OFFSET..]
            };
            <T as PrimitiveRead>::try_read::<E>(&storage)
        }
    }

    doc_comment::doc_comment! {
        concat! {"
        Write the integer field to a given data region, assuming the defined layout, using the [Field] API.

        # Example:

        ```
        use binary_layout::prelude::*;
        use core::convert::Infallible;

        binary_layout!(my_layout, LittleEndian, {
            //... other fields ...
            some_integer_field: u32,
            //... other fields ...
        });

        fn func(storage_data: &mut [u8]) {
            my_layout::some_integer_field::try_write(storage_data, 10).unwrap();
        }
        ```
        "},
        #[inline(always)]
        fn try_write(storage: &mut [u8], value: T) -> Result<(), Self::WriteError> {
            let storage = if let Some(size) = Self::SIZE {
                &mut storage[Self::OFFSET..(Self::OFFSET + size)]
            } else {
                &mut storage[Self::OFFSET..]
            };
            <T as PrimitiveWrite>::try_write::<E>(value, storage)
        }
    }
}

mod read_write_ext;

pub use read_write_ext::{FieldReadExt, FieldWriteExt};
