use super::{PrimitiveRead, PrimitiveWrite};
use crate::data_types::DataTypeMetadata;
use crate::endianness::{EndianKind, Endianness};
use crate::view::PrimitiveFieldView;
use crate::Field;

macro_rules! impl_float {
    ($type:ty) => {
        impl DataTypeMetadata for $type {
            const SIZE: Option<usize> = Some(core::mem::size_of::<$type>());

            type View<S, F> = PrimitiveFieldView<S, F> where F: Field;
        }

        impl PrimitiveRead for $type {
            /// Reading floats can't fail
            type Error = core::convert::Infallible;

            doc_comment::doc_comment! {
                concat! {"
                Read the float field from a given storage.
                The storage slice size must exactly match the size of the expected float, otherwise this will panic.

                # Warning
                At it's core, this method uses [", stringify!($type), "::from_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.from_bits),
                which has some weird behavior around signaling and non-signaling `NaN` values.  Read the
                documentation for [", stringify!($type), "::from_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.from_bits) which
                explains the situation.
                "},
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
        }

        impl PrimitiveWrite for $type {
            /// Writing floats can't fail
            type Error = core::convert::Infallible;

            doc_comment::doc_comment! {
                concat! {"
                Write the float field to a given storage.
                The storage slice size must exactly match the size of the expected float, otherwise this will panic.

                # WARNING
                At it's core, this method uses [", stringify!($type), "::to_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.to_bits),
                which has some weird behavior around signaling and non-signaling `NaN` values.  Read the
                documentation for [", stringify!($type), "::to_bits](https://doc.rust-lang.org/std/primitive.", stringify!($type), ".html#method.to_bits) which
                explains the situation.
                "},
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
        }
    };
}

impl_float!(f32);
impl_float!(f64);

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
                fn [<test_ $type _ $endian endian_fieldapi_read_write>]() {
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
                fn [<test_ $type _ $endian endian_fieldapi_tryread_trywrite>]() {
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
                    view.field3_mut().write(0.0);

                    assert_eq!($value1, view.field1().read());
                    assert_eq!($value2, view.field2().read());
                    assert_eq!(0.0, view.field3().read());

                    assert_eq!($value1, $type::$endian_fn((&storage[0..(0+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[$expected_size..(2*$expected_size)]).try_into().unwrap()));
                    assert_eq!(0.0, $type::$endian_fn((&storage[2*$expected_size..(3*$expected_size)]).try_into().unwrap()));
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
                    view.field3_mut().try_write(0.0).infallible_unwrap();

                    assert_eq!($value1, view.field1().try_read().infallible_unwrap());
                    assert_eq!($value2, view.field2().try_read().infallible_unwrap());
                    assert_eq!(0.0, view.field3().try_read().infallible_unwrap());

                    assert_eq!($value1, $type::$endian_fn((&storage[0..(0+$expected_size)]).try_into().unwrap()));
                    assert_eq!($value2, $type::$endian_fn((&storage[$expected_size..(2*$expected_size)]).try_into().unwrap()));
                    assert_eq!(0.0, $type::$endian_fn((&storage[2*$expected_size..(3*$expected_size)]).try_into().unwrap()));
                }
            }
        };
    }

    test_float!(f32, 4, 10f32.powf(8.31), -(10f32.powf(7.31)));
    test_float!(f64, 8, 10f64.powf(15.31), -(10f64.powf(15.31)));
}
