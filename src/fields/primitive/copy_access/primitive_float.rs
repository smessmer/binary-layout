use core::convert::Infallible;

use super::{FieldCopyAccess, PrimitiveField};
use crate::endianness::{EndianKind, Endianness};
use crate::fields::primitive::view::FieldView;
use crate::fields::{Field, StorageIntoFieldView, StorageToFieldView};

macro_rules! float_field {
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
                Read the float field from a given data region, assuming the defined layout, using the [Field] API.

                # Example:

                ```
                use binary_layout::prelude::*;
                use core::convert::Infallible;

                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_float_field: ", stringify!($type), "
                    //... other fields ...
                });

                fn func(storage_data: &[u8]) -> ", stringify!($type), " {
                    let read: ", stringify!($type), " = my_layout::some_float_field::try_read(storage_data).unwrap();
                    read
                }
                ```

                # WARNING

                At it's core, this method uses [", stringify!($type), "::from_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.from_bits),
                which has some weird behavior around signaling and non-signaling `NaN` values.  Read the
                documentation for [", stringify!($type), "::from_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.from_bits) which
                explains the situation.
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
                        EndianKind::Native => <$type>::from_ne_bytes(value),
                    };
                    Ok(value)
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
                    my_layout::some_float_field::try_write(storage_data, 10.0).unwrap();
                }
                ```

                # WARNING

                At it's core, this method uses [", stringify!($type), "::to_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.to_bits),
                which has some weird behavior around signaling and non-signaling `NaN` values.  Read the
                documentation for [", stringify!($type), "::to_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.to_bits) which
                explains the situation.
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

float_field!(f32);
float_field!(f64);

// TODO Tests (I think we accidentally deleted them a few commits further up when we removed the previous version of FieldCopyAccess and replaced it with a version that is based on FieldTryCopyAccess)
