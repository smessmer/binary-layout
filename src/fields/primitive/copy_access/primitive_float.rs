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
                    let value: [u8; core::mem::size_of::<$type>()] = storage[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())].try_into().unwrap();
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

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use crate::prelude::*;
    use crate::PrimitiveField;

    macro_rules! test_float {
        ($type:ty, $expected_size:expr, $value1:expr, $value2:expr) => {
            test_float!(@case, $type, $expected_size, $value1, $value2, little, LittleEndian, from_le_bytes);
            test_float!(@case, $type, $expected_size, $value1, $value2, big, BigEndian, from_be_bytes);
            test_float!(@case, $type, $expected_size, $value1, $value2, native, NativeEndian, from_ne_bytes);
        };
        (@case, $type:ty, $expected_size:expr, $value1:expr, $value2: expr, $endian:ident, $endian_type:ty, $endian_fn:ident) => {
            $crate::internal::paste! {
                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_metadata>]() {
                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;
                    type Field3 = PrimitiveField<$type, $endian_type, 150>;

                    assert_eq!(Some($expected_size), Field1::SIZE);
                    assert_eq!(5, Field1::OFFSET);
                    assert_eq!(Some($expected_size), Field2::SIZE);
                    assert_eq!(123, Field2::OFFSET);
                    assert_eq!(Some($expected_size), Field3::SIZE);
                    assert_eq!(150, Field3::OFFSET);
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_read_write>]() {
                    let mut storage = [0; 1024];

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;
                    type Field3 = PrimitiveField<$type, $endian_type, 150>;

                    Field1::write(&mut storage, $value1);
                    Field2::write(&mut storage, $value2);
                    Field3::write(&mut storage, 0.0);

                    assert_eq!($value1, Field1::read(&storage));
                    assert_eq!($value2, Field2::read(&storage));
                    assert_eq!(0.0, Field3::read(&storage));

                    assert_eq!($value1, $type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap()));
                    assert_eq!(0.0, $type::$endian_fn((&storage[150..(150+$expected_size)]).try_into().unwrap()));
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_tryread_trywrite>]() {
                    use crate::InfallibleResultExt;

                    let mut storage = [0; 1024];

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;
                    type Field3 = PrimitiveField<$type, $endian_type, 150>;

                    Field1::try_write(&mut storage, $value1).infallible_unwrap();
                    Field2::try_write(&mut storage, $value2).infallible_unwrap();
                    Field3::try_write(&mut storage, 0.0).infallible_unwrap();

                    assert_eq!($value1, Field1::try_read(&storage).infallible_unwrap());
                    assert_eq!($value2, Field2::try_read(&storage).infallible_unwrap());
                    assert_eq!(0.0, Field3::try_read(&storage).infallible_unwrap());

                    assert_eq!($value1, $type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap()));
                    assert_eq!(0.0, $type::$endian_fn((&storage[150..(150+$expected_size)]).try_into().unwrap()));
                }
            }
        };
    }

    test_float!(f32, 4, 10f32.powf(8.31), -(10f32.powf(7.31)));
    test_float!(f64, 8, 10f64.powf(15.31), -(10f64.powf(15.31)));
}
