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
                    // TODO Don't initialize memory
                    let mut value = [0; core::mem::size_of::<$type>()];
                    value.copy_from_slice(
                        &storage[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())],
                    );
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

// TODO Tests (I think we accidentally deleted them a few commits further up when we removed the previous version of FieldCopyAccess and replaced it with a version that is based on FieldTryCopyAccess)
