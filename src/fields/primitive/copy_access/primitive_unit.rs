use core::convert::Infallible;

use super::{FieldCopyAccess, PrimitiveField};
use crate::endianness::Endianness;
use crate::fields::primitive::view::FieldView;
use crate::fields::{Field, StorageIntoFieldView, StorageToFieldView};

impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess for PrimitiveField<(), E, OFFSET_> {
    /// See [FieldCopyAccess::ReadError]
    type ReadError = Infallible;
    /// See [FieldCopyAccess::WriteError]
    type WriteError = Infallible;
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
                    let read: ", stringify!(()), " = my_layout::some_zst_field::try_read(storage_data).unwrap();
                    read
                }
                ```

                In reality, this method doesn't do any work; `",
                stringify!(()), "` is a zero-sized type, so there's no work to
                do. This implementation exists solely to make writing derive
                macros simpler.
                "},
        #[inline(always)]
        #[allow(clippy::unused_unit)] // I don't want to remove this as it's part of the trait.
        fn try_read(_storage: &[u8]) -> Result<(), Infallible> {
            Ok(())
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
                    my_layout::some_zst_field::try_write(storage_data, ()).unwrap();
                }
                ```

                # WARNING

                In reality, this method doesn't do any work; `",
                stringify!(()), "` is a zero-sized type, so there's no work to
                do. This implementation exists solely to make writing derive
                macros simpler.
                "},
        #[inline(always)]
        #[allow(clippy::unused_unit)] // I don't want to remove this as it's part of the trait.
        fn try_write(_storage: &mut [u8], _value: ()) -> Result<(), Infallible> {
            Ok(())
        }
    }
}

impl_field_traits!(());

// TODO Tests (I think we accidentally deleted them a few commits further up when we removed the previous version of FieldCopyAccess and replaced it with a version that is based on FieldTryCopyAccess)
