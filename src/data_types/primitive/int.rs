use super::{PrimitiveRead, PrimitiveWrite};
use crate::data_types::DataTypeMetadata;
use crate::endianness::{EndianKind, Endianness};
use crate::view::PrimitiveFieldView;
use crate::Field;

macro_rules! impl_int {
    ($type:ty) => {
        impl DataTypeMetadata for $type {
            const SIZE: Option<usize> = Some(core::mem::size_of::<$type>());

            type View<S, F> = PrimitiveFieldView<S, F> where F: Field;
        }

        impl PrimitiveRead for $type {
            /// Reading integers can't fail
            type Error = core::convert::Infallible;

            /// Read the integer field from a given storage.
            /// The storage slice size must exactly match the size of the expected integer, otherwise this will panic.
            #[inline(always)]
            fn try_read<E: Endianness>(storage: &[u8]) -> Result<Self, Self::Error> {
                let value: [u8; core::mem::size_of::<$type>()] = storage.try_into().unwrap();
                let value = match E::KIND {
                    EndianKind::Big => <$type>::from_be_bytes(value),
                    EndianKind::Little => <$type>::from_le_bytes(value),
                    EndianKind::Native => <$type>::from_ne_bytes(value),
                };
                Ok(value)
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
                    EndianKind::Big => self.to_be_bytes(),
                    EndianKind::Little => self.to_le_bytes(),
                    EndianKind::Native => self.to_ne_bytes(),
                };
                storage.copy_from_slice(&value);
                Ok(())
            }
        }
    };
}

impl_int!(u8);
impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(u128);
impl_int!(i8);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);
impl_int!(i128);

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::PrimitiveField;

    macro_rules! test_int {
        ($type:ty, $expected_size:expr, $value1:expr, $value2:expr) => {
            test_int!(@case, $type, $expected_size, $value1, $value2, little, LittleEndian, from_le_bytes);
            test_int!(@case, $type, $expected_size, $value1, $value2, big, BigEndian, from_be_bytes);
            test_int!(@case, $type, $expected_size, $value1, $value2, native, NativeEndian, from_ne_bytes);
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
                fn [<test_ $type _ $endian endian_fieldapi_read_write>]() {
                    let mut storage = [0; 1024];

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;
                    type Field3 = PrimitiveField<$type, $endian_type, 150>;

                    Field1::write(&mut storage, $value1);
                    Field2::write(&mut storage, $value2);
                    Field3::write(&mut storage, 0);

                    assert_eq!($value1, Field1::read(&storage));
                    assert_eq!($value2, Field2::read(&storage));
                    assert_eq!(0, Field3::read(&storage));

                    assert_eq!($value1, $type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap()));
                    assert_eq!(0, $type::$endian_fn((&storage[150..(150+$expected_size)]).try_into().unwrap()));
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_fieldapi_tryread_trywrite>]() {
                    use crate::InfallibleResultExt;

                    let mut storage = [0; 1024];

                    type Field1 = PrimitiveField<$type, $endian_type, 5>;
                    type Field2 = PrimitiveField<$type, $endian_type, 123>;
                    type Field3 = PrimitiveField<$type, $endian_type, 150>;

                    Field1::try_write(&mut storage, $value1).infallible_unwrap();
                    Field2::try_write(&mut storage, $value2).infallible_unwrap();
                    Field3::try_write(&mut storage, 0).infallible_unwrap();

                    assert_eq!($value1, Field1::try_read(&storage).infallible_unwrap());
                    assert_eq!($value2, Field2::try_read(&storage).infallible_unwrap());
                    assert_eq!(0, Field3::try_read(&storage).infallible_unwrap());

                    assert_eq!($value1, $type::$endian_fn((&storage[5..(5+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[123..(123+$expected_size)]).try_into().unwrap()));
                    assert_eq!(0, $type::$endian_fn((&storage[150..(150+$expected_size)]).try_into().unwrap()));
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_ $type _ $endian endian_viewapi_read_write>]() {
                    binary_layout!(layout, $endian_type, {
                        field1: $type,
                        field2: $type,
                        field3: $type,
                    });
                    let mut storage = [0; 1024];
                    let mut view = layout::View::new(&mut storage);

                    view.field1_mut().write($value1);
                    view.field2_mut().write($value2);
                    view.field3_mut().write(0);

                    assert_eq!($value1, view.field1().read());
                    assert_eq!($value2, view.field2().read());
                    assert_eq!(0, view.field3().read());

                    assert_eq!($value1, $type::$endian_fn((&storage[0..(0+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[$expected_size..(2*$expected_size)]).try_into().unwrap()));
                    assert_eq!(0, $type::$endian_fn((&storage[2*$expected_size..(3*$expected_size)]).try_into().unwrap()));
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

                    view.field1_mut().try_write($value1).infallible_unwrap();
                    view.field2_mut().try_write($value2).infallible_unwrap();
                    view.field3_mut().try_write(0).infallible_unwrap();

                    assert_eq!($value1, view.field1().try_read().infallible_unwrap());
                    assert_eq!($value2, view.field2().try_read().infallible_unwrap());
                    assert_eq!(0, view.field3().try_read().infallible_unwrap());

                    assert_eq!($value1, $type::$endian_fn((&storage[0..(0+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[$expected_size..(2*$expected_size)]).try_into().unwrap()));
                    assert_eq!(0, $type::$endian_fn((&storage[2*$expected_size..(3*$expected_size)]).try_into().unwrap()));
                }
            }
        };
    }

    test_int!(i8, 1, 50, -20);
    test_int!(i16, 2, 500, -2000);
    test_int!(i32, 4, 10i32.pow(8), -(10i32.pow(7)));
    test_int!(i64, 8, 10i64.pow(15), -(10i64.pow(14)));
    test_int!(i128, 16, 10i128.pow(30), -(10i128.pow(28)));

    test_int!(u8, 1, 50, 20);
    test_int!(u16, 2, 500, 2000);
    test_int!(u32, 4, 10u32.pow(8), (10u32.pow(7)));
    test_int!(u64, 8, 10u64.pow(15), (10u64.pow(14)));
    test_int!(u128, 16, 10u128.pow(30), (10u128.pow(28)));
}
