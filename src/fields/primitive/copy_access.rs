use super::super::{Field, StorageIntoFieldView, StorageToFieldView};
use super::view::FieldView;
use super::PrimitiveField;
use crate::endianness::{EndianKind, Endianness};

/// This trait is implemented for fields with "copy access",
/// i.e. fields that read/write data by copying it from/to the
/// binary blob. Examples of this are primitive types
/// like u8, i32, ...
pub trait FieldCopyAccess: Field {
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

macro_rules! int_field {
    ($type:ty) => {
        impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess for PrimitiveField<$type, E, OFFSET_> {
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

                fn func(storage_data: &[u8]) {
                    let read: ", stringify!($type), " = my_layout::some_integer_field::read(storage_data);
                }
                ```
                "},
                #[inline(always)]
                fn read(storage: &[u8]) -> $type {
                    // TODO Don't initialize memory
                    let mut value = [0; core::mem::size_of::<$type>()];
                    value.copy_from_slice(
                        &storage[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())],
                    );
                    match E::KIND {
                        EndianKind::Big => <$type>::from_be_bytes(value),
                        EndianKind::Little => <$type>::from_le_bytes(value),
                        EndianKind::Native => <$type>::from_ne_bytes(value)
                    }
                }
            }

            doc_comment::doc_comment! {
                concat! {"
                Write the integer field to a given data region, assuming the defined layout, using the [Field] API.

                # Example:

                ```
                use binary_layout::prelude::*;

                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_integer_field: ", stringify!($type), "
                    //... other fields ...
                });

                fn func(storage_data: &mut [u8]) {
                    my_layout::some_integer_field::write(storage_data, 10);
                }
                ```
                "},
                #[inline(always)]
                fn write(storage: &mut [u8], value: $type) {
                    let value_as_bytes = match E::KIND {
                        EndianKind::Big => value.to_be_bytes(),
                        EndianKind::Little => value.to_le_bytes(),
                        EndianKind::Native => value.to_ne_bytes(),
                    };
                    storage[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())]
                        .copy_from_slice(&value_as_bytes);
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

macro_rules! float_field {
    ($type:ty) => {
        impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess for PrimitiveField<$type, E, OFFSET_> {
            /// See [FieldCopyAccess::HighLevelType]
            type HighLevelType = $type;

            doc_comment::doc_comment! {
                concat! {"
                Read the float field from a given data region, assuming the defined layout, using the [Field] API.

                # Example:

                ```
                use binary_layout::prelude::*;

                define_layout!(my_layout, LittleEndian, {
                    //... other fields ...
                    some_float_field: ", stringify!($type), "
                    //... other fields ...
                });

                fn func(storage_data: &[u8]) {
                    let read: ", stringify!($type), " = my_layout::some_float_field::read(storage_data);
                }
                ```

                # WARNING

                At it's core, this method uses [", stringify!($type), "::from_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.from_bits),
                which has some weird behavior around signaling and non-signaling `NaN` values.  Read the
                documentation for [", stringify!($type), "::from_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.from_bits) which
                explains the situation.
                "},
                #[inline(always)]
                fn read(storage: &[u8]) -> $type {
                    // TODO Don't initialize memory
                    let mut value = [0; core::mem::size_of::<$type>()];
                    value.copy_from_slice(
                        &storage[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())],
                    );
                    match E::KIND {
                        EndianKind::Big => <$type>::from_be_bytes(value),
                        EndianKind::Little => <$type>::from_le_bytes(value),
                        EndianKind::Native => <$type>::from_ne_bytes(value),
                    }
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
                    my_layout::some_float_field::write(storage_data, 10.0);
                }
                ```

                # WARNING

                At it's core, this method uses [", stringify!($type), "::to_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.to_bits),
                which has some weird behavior around signaling and non-signaling `NaN` values.  Read the
                documentation for [", stringify!($type), "::to_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.to_bits) which
                explains the situation.
                "},
                #[inline(always)]
                fn write(storage: &mut [u8], value: $type) {
                    let value_as_bytes = match E::KIND {
                        EndianKind::Big => value.to_be_bytes(),
                        EndianKind::Little => value.to_le_bytes(),
                        EndianKind::Native => value.to_ne_bytes(),
                    };
                    storage[Self::OFFSET..(Self::OFFSET + core::mem::size_of::<$type>())]
                        .copy_from_slice(&value_as_bytes);
                }
            }
        }

        impl_field_traits!($type);
    };
}

float_field!(f32);
float_field!(f64);

impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess for PrimitiveField<(), E, OFFSET_> {
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
                    let read: ", stringify!(()), " = my_layout::some_zst_field::read(storage_data);
                }
                ```

                In reality, this method doesn't do any work; `",
                stringify!(()), "` is a zero-sized type, so there's no work to
                do. This implementation exists solely to make writing derive
                macros simpler.
                "},
        #[inline(always)]
        #[allow(clippy::unused_unit)] // I don't want to remove this as it's part of the trait.
        fn read(_storage: &[u8]) -> () {
            ()
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
                    my_layout::some_zst_field::write(storage_data, ());
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
        fn write(_storage: &mut [u8], _value: ()) {
            ()
        }
    }
}

impl_field_traits!(());

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
                    let mut storage = vec![0; 1024];

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
                    let mut storage = vec![0; 1024];

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
                    assert_eq!(storage, vec![0; 1024]);
                }
            }
        };
    }

    test_unit_copy_access!(little, LittleEndian);
    test_unit_copy_access!(big, BigEndian);
    test_unit_copy_access!(native, NativeEndian);
}
