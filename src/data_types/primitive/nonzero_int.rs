use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

use super::{PrimitiveRead, PrimitiveWrite};
use crate::data_types::DataTypeMetadata;
use crate::endianness::{EndianKind, Endianness};
use crate::view::PrimitiveFieldView;
use crate::Field;

/// This error is thrown when trying to read a non-zero integer type, e.g. [NonZeroU32](core::num::NonZeroU32),
/// but the data being read was actually zero.
#[derive(Debug)]
pub struct NonZeroIsZeroError(pub(crate) ());

impl core::fmt::Display for NonZeroIsZeroError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "NonZeroIsZeroError")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NonZeroIsZeroError {}

macro_rules! impl_nonzero_int {
    ($type:ty, $zeroable_type:ty) => {
        impl DataTypeMetadata for $type {
            const SIZE: Option<usize> = Some(core::mem::size_of::<$type>());

            type View<S, F> = PrimitiveFieldView<S, F> where F: Field;
        }

        impl PrimitiveRead for $type {
            /// Reading integers can fail if a zero is read
            type Error = NonZeroIsZeroError;

            /// Read the integer field from a given storage.
            /// The storage slice size must exactly match the size of the expected integer, otherwise this will panic.
            #[inline(always)]
            fn try_read<E: Endianness>(storage: &[u8]) -> Result<Self, Self::Error> {
                let value: [u8; core::mem::size_of::<$type>()] = storage.try_into().unwrap();
                let value = match E::KIND {
                    EndianKind::Big => <$zeroable_type>::from_be_bytes(value),
                    EndianKind::Little => <$zeroable_type>::from_le_bytes(value),
                    EndianKind::Native => <$zeroable_type>::from_ne_bytes(value),
                };
                <$type>::new(value).ok_or(NonZeroIsZeroError(()))
            }
        }

        impl PrimitiveWrite for $type {
            /// Writing integers can't fail
            type Error = core::convert::Infallible;

            /// Write the integer field to a given storage.
            /// The storage slice size must exactly match the size of the expected integer, otherwise this will panic.
            #[inline(always)]
            fn try_write<E: Endianness>(self, storage: &mut [u8]) -> Result<(), Self::Error> {
                let value = match E::KIND {
                    EndianKind::Big => self.get().to_be_bytes(),
                    EndianKind::Little => self.get().to_le_bytes(),
                    EndianKind::Native => self.get().to_ne_bytes(),
                };
                storage.copy_from_slice(&value);
                Ok(())
            }
        }
    };
}

impl_nonzero_int!(NonZeroU8, u8);
impl_nonzero_int!(NonZeroU16, u16);
impl_nonzero_int!(NonZeroU32, u32);
impl_nonzero_int!(NonZeroU64, u64);
impl_nonzero_int!(NonZeroU128, u128);
impl_nonzero_int!(NonZeroI8, i8);
impl_nonzero_int!(NonZeroI16, i16);
impl_nonzero_int!(NonZeroI32, i32);
impl_nonzero_int!(NonZeroI64, i64);
impl_nonzero_int!(NonZeroI128, i128);

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::PrimitiveField;

    macro_rules! test_nonzero {
        ($type:ty, $underlying_type:ty, $expected_size:expr, $value1:expr, $value2:expr) => {
            test_nonzero!(@case, $type, $underlying_type, $expected_size, $value1, $value2, little, LittleEndian, from_le_bytes);
            test_nonzero!(@case, $type, $underlying_type, $expected_size, $value1, $value2, big, BigEndian, from_be_bytes);
            test_nonzero!(@case, $type, $underlying_type, $expected_size, $value1, $value2, native, NativeEndian, from_ne_bytes);
        };
        (@case, $type:ty, $underlying_type:ty, $expected_size:expr, $value1:expr, $value2: expr, $endian:ident, $endian_type:ty, $endian_fn:ident) => {
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
                fn [<test_ $type _ $endian endian_fieldapi_tryread_write>]() {
                    let mut storage = [0; 1024];

                    let value1 = <$type>::new($value1).unwrap();
                    let value2 = <$type>::new($value2).unwrap();

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;
                    type Field3 = PrimitiveField<$type, $endian_type, 150>;

                    Field1::write(&mut storage, value1);
                    Field2::write(&mut storage, value2);
                    // don't write Field3, that should leave it at zero

                    assert_eq!(value1, Field1::try_read(&storage).unwrap());
                    assert_eq!(value2, Field2::try_read(&storage).unwrap());
                    assert!(matches!(Field3::try_read(&storage), Err(NonZeroIsZeroError(_))));

                    assert_eq!(value1, $type::new($underlying_type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(value2, $type::new($underlying_type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(0, $underlying_type::$endian_fn((&storage[150..(150+$expected_size)]).try_into().unwrap()));
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_fieldapi_tryread_trywrite>]() {
                    use crate::InfallibleResultExt;

                    let mut storage = [0; 1024];

                    let value1 = <$type>::new($value1).unwrap();
                    let value2 = <$type>::new($value2).unwrap();

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;
                    type Field3 = PrimitiveField<$type, $endian_type, 150>;

                    Field1::try_write(&mut storage, value1).infallible_unwrap();
                    Field2::try_write(&mut storage, value2).infallible_unwrap();
                    // don't write Field3, that should leave it at zero

                    assert_eq!(value1, Field1::try_read(&storage).unwrap());
                    assert_eq!(value2, Field2::try_read(&storage).unwrap());
                    assert!(matches!(Field3::try_read(&storage), Err(NonZeroIsZeroError(_))));

                    assert_eq!(value1, $type::new($underlying_type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(value2, $type::new($underlying_type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(0, $underlying_type::$endian_fn((&storage[150..(150+$expected_size)]).try_into().unwrap()));
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_viewapi_tryread_write>]() {
                    binary_layout!(layout, $endian_type, {
                        field1: $type,
                        field2: $type,
                        field3: $type,
                    });
                    let mut storage = [0; 1024];
                    let mut view = layout::View::new(&mut storage);

                    let value1 = <$type>::new($value1).unwrap();
                    let value2 = <$type>::new($value2).unwrap();

                    view.field1_mut().write(value1);
                    view.field2_mut().write(value2);
                    // don't write Field3, that should leave it at zero

                    assert_eq!(value1, view.field1().try_read().unwrap());
                    assert_eq!(value2, view.field2().try_read().unwrap());
                    assert!(matches!(view.field3().try_read(), Err(NonZeroIsZeroError(_))));

                    assert_eq!(value1, $type::new($underlying_type::$endian_fn((&storage[0..($expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(value2, $type::new($underlying_type::$endian_fn((&storage[$expected_size..(2*$expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(0, $underlying_type::$endian_fn((&storage[2*$expected_size..(3*$expected_size)]).try_into().unwrap()));
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_viewapi_tryread_trywrite>]() {
                    binary_layout!(layout, $endian_type, {
                        field1: $type,
                        field2: $type,
                        field3: $type,
                    });
                    let mut storage = [0; 1024];
                    let mut view = layout::View::new(&mut storage);

                    let value1 = <$type>::new($value1).unwrap();
                    let value2 = <$type>::new($value2).unwrap();

                    view.field1_mut().try_write(value1).infallible_unwrap();
                    view.field2_mut().try_write(value2).infallible_unwrap();
                    // don't write Field3, that should leave it at zero

                    assert_eq!(value1, view.field1().try_read().unwrap());
                    assert_eq!(value2, view.field2().try_read().unwrap());
                    assert!(matches!(view.field3().try_read(), Err(NonZeroIsZeroError(_))));

                    assert_eq!(value1, $type::new($underlying_type::$endian_fn((&storage[0..($expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(value2, $type::new($underlying_type::$endian_fn((&storage[$expected_size..(2*$expected_size)]).try_into().unwrap())).unwrap());
                    assert_eq!(0, $underlying_type::$endian_fn((&storage[2*$expected_size..(3*$expected_size)]).try_into().unwrap()));
                }
            }
        };
    }

    use core::num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    };

    test_nonzero!(NonZeroI8, i8, 1, 50, -20);
    test_nonzero!(NonZeroI16, i16, 2, 500, -2000);
    test_nonzero!(NonZeroI32, i32, 4, 10i32.pow(8), -(10i32.pow(7)));
    test_nonzero!(NonZeroI64, i64, 8, 10i64.pow(15), -(10i64.pow(14)));
    test_nonzero!(NonZeroI128, i128, 16, 10i128.pow(30), -(10i128.pow(28)));

    test_nonzero!(NonZeroU8, u8, 1, 50, 20);
    test_nonzero!(NonZeroU16, u16, 2, 500, 2000);
    test_nonzero!(NonZeroU32, u32, 4, 10u32.pow(8), (10u32.pow(7)));
    test_nonzero!(NonZeroU64, u64, 8, 10u64.pow(15), (10u64.pow(14)));
    test_nonzero!(NonZeroU128, u128, 16, 10u128.pow(30), (10u128.pow(28)));
}
