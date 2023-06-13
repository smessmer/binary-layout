use core::marker::PhantomData;

use super::{
    primitive::{FieldCopyAccess, FieldView},
    Field, StorageIntoFieldView, StorageToFieldView,
};

/// Implementing the [LayoutAs] trait for a custom type allows that custom type to be used
/// as the type of a layout field. Note that the value of this type is copied each time it
/// is accessed, so this is only recommended for primitive wrappers of primitive types,
/// not for types that are expensive to copy.
///
/// # Example
/// ```
/// use binary_layout::{prelude::*, LayoutAs};
///
/// struct MyIdType(u64);
/// impl LayoutAs<u64> for MyIdType {
///   fn read(v: u64) -> MyIdType {
///     MyIdType(v)
///   }
///
///   fn write(v: MyIdType) -> u64 {
///     v.0
///   }
/// }
///
/// define_layout!(my_layout, BigEndian, {
///   // ... other fields ...
///   field: MyIdType as u64,
///   // ... other fields ...
/// });
///
/// # fn main() {}
/// ```
pub trait LayoutAs<U> {
    /// Implement this to define how the custom type is constructed from the underlying type
    /// after it was read from a layouted binary slice.
    fn read(v: U) -> Self;

    /// Implement this to define how the custom type is converted into the underlying type
    /// so it can be written into a layouted binary slice.
    fn write(v: Self) -> U;
}

/// A [WrappedField] is a [Field] that, unlike [PrimitiveField](crate::PrimitiveField), does not directly represent a primitive type.
/// Instead, it represents a wrapper type that can be converted to/from a primitive type using the [LayoutAs] trait.
/// See [Field] for more info on this API.
///
/// # Example:
/// ```
/// use binary_layout::{prelude::*, LayoutAs};
///
/// struct MyIdType(u64);
/// impl LayoutAs<u64> for MyIdType {
///   fn read(v: u64) -> MyIdType {
///     MyIdType(v)
///   }
///
///   fn write(v: MyIdType) -> u64 {
///     v.0
///   }
/// }
///
/// define_layout!(my_layout, BigEndian, {
///   // ... other fields ...
///   field: MyIdType as u64,
///   // ... other fields ...
/// });
///
/// fn func(storage_data: &mut [u8]) {
///   // read some data
///   let read_data: MyIdType = my_layout::field::read(storage_data);
///   // equivalent: let read_data = MyIdType(u16::from_le_bytes((&storage_data[0..2]).try_into().unwrap()));
///
///   // write some data
///   my_layout::field::write(storage_data, MyIdType(10));
///   // equivalent: data_slice[18..22].copy_from_slice(&10u32.to_le_bytes());
/// }
///
/// # fn main() {
/// #   let mut storage = [0; 1024];
/// #   func(&mut storage);
/// # }
/// ```
pub struct WrappedField<U, T: LayoutAs<U>, F: Field> {
    _p1: PhantomData<U>,
    _p2: PhantomData<T>,
    _p3: PhantomData<F>,
}

impl<U, T: LayoutAs<U>, F: Field> Field for WrappedField<U, T, F> {
    /// See [Field::Endian]
    type Endian = F::Endian;
    /// See [Field::OFFSET]
    const OFFSET: usize = F::OFFSET;
    /// See [Field::SIZE]
    const SIZE: Option<usize> = F::SIZE;
}

impl<
        'a,
        U,
        T: LayoutAs<U>,
        F: FieldCopyAccess<HighLevelType = U> + StorageToFieldView<&'a [u8]>,
    > StorageToFieldView<&'a [u8]> for WrappedField<U, T, F>
{
    type View = FieldView<&'a [u8], Self>;

    #[inline(always)]
    fn view(storage: &'a [u8]) -> Self::View {
        Self::View::new(storage)
    }
}

impl<
        'a,
        U,
        T: LayoutAs<U>,
        F: FieldCopyAccess<HighLevelType = U> + StorageToFieldView<&'a mut [u8]>,
    > StorageToFieldView<&'a mut [u8]> for WrappedField<U, T, F>
{
    type View = FieldView<&'a mut [u8], Self>;

    #[inline(always)]
    fn view(storage: &'a mut [u8]) -> Self::View {
        Self::View::new(storage)
    }
}

impl<
        U,
        S: AsRef<[u8]>,
        T: LayoutAs<U>,
        F: FieldCopyAccess<HighLevelType = U> + StorageIntoFieldView<S>,
    > StorageIntoFieldView<S> for WrappedField<U, T, F>
{
    type View = FieldView<S, Self>;

    #[inline(always)]
    fn into_view(storage: S) -> Self::View {
        Self::View::new(storage)
    }
}

impl<U, T: LayoutAs<U>, F: FieldCopyAccess<HighLevelType = U>> FieldCopyAccess
    for WrappedField<U, T, F>
{
    /// See [FieldCopyAccess::HighLevelType]
    type HighLevelType = T;

    /// Read the field from a given data region, assuming the defined layout, using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::{prelude::*, LayoutAs};
    ///
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct MyIdType(u64);
    /// impl LayoutAs<u64> for MyIdType {
    ///   fn read(v: u64) -> MyIdType {
    ///     MyIdType(v)
    ///   }
    ///
    ///   fn write(v: MyIdType) -> u64 {
    ///     v.0
    ///   }
    /// }
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   //... other fields ...
    ///   some_integer_field: MyIdType as u64,
    ///   //... other fields ...
    /// });
    ///
    /// fn func(storage_data: &mut [u8]) {
    ///   my_layout::some_integer_field::write(storage_data, MyIdType(50));
    ///   assert_eq!(MyIdType(50), my_layout::some_integer_field::read(storage_data));
    /// }
    ///
    /// # fn main() {
    /// #   let mut storage = [0; 1024];
    /// #   func(&mut storage);
    /// # }
    /// ```
    #[inline(always)]
    fn read(storage: &[u8]) -> Self::HighLevelType {
        let v = F::read(storage);
        <T as LayoutAs<U>>::read(v)
    }

    /// Write the field to a given data region, assuming the defined layout, using the [Field] API.
    ///
    /// # Example:
    /// See [FieldCopyAccess::read] for an example
    #[inline(always)]
    fn write(storage: &mut [u8], v: Self::HighLevelType) {
        let v = <T as LayoutAs<U>>::write(v);
        F::write(storage, v)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use crate::prelude::*;
    use crate::{LayoutAs, PrimitiveField, WrappedField};
    use core::convert::TryInto;

    #[derive(Debug, PartialEq, Eq)]
    struct Wrapped<T>(T);
    impl<T> LayoutAs<T> for Wrapped<T> {
        fn read(v: T) -> Self {
            Self(v)
        }
        fn write(v: Self) -> T {
            v.0
        }
    }

    #[test]
    fn test_i8_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i8, Wrapped<i8>, PrimitiveField<i8, LittleEndian, 5>>;
        type Field2 = WrappedField<i8, Wrapped<i8>, PrimitiveField<i8, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(50));
        Field2::write(&mut storage, Wrapped(-20));

        assert_eq!(Wrapped(50), Field1::read(&storage));
        assert_eq!(Wrapped(-20), Field2::read(&storage));

        assert_eq!(50, i8::from_le_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            -20,
            i8::from_le_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(
            Some(1),
            WrappedField::<i8, Wrapped<i8>, PrimitiveField::<i8, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(1),
            WrappedField::<i8, Wrapped<i8>, PrimitiveField::<i8, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i8_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i8, Wrapped<i8>, PrimitiveField<i8, BigEndian, 5>>;
        type Field2 = WrappedField<i8, Wrapped<i8>, PrimitiveField<i8, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(50));
        Field2::write(&mut storage, Wrapped(-20));

        assert_eq!(Wrapped(50), Field1::read(&storage));
        assert_eq!(Wrapped(-20), Field2::read(&storage));

        assert_eq!(50, i8::from_be_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            -20,
            i8::from_be_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(
            Some(1),
            WrappedField::<i8, Wrapped<i8>, PrimitiveField::<i8, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(1),
            WrappedField::<i8, Wrapped<i8>, PrimitiveField::<i8, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i8_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i8, Wrapped<i8>, PrimitiveField<i8, NativeEndian, 5>>;
        type Field2 = WrappedField<i8, Wrapped<i8>, PrimitiveField<i8, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(50));
        Field2::write(&mut storage, Wrapped(-20));

        assert_eq!(Wrapped(50), Field1::read(&storage));
        assert_eq!(Wrapped(-20), Field2::read(&storage));

        assert_eq!(50, i8::from_ne_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            -20,
            i8::from_ne_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(
            Some(1),
            WrappedField::<i8, Wrapped<i8>, PrimitiveField::<i8, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(1),
            WrappedField::<i8, Wrapped<i8>, PrimitiveField::<i8, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i16_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i16, Wrapped<i16>, PrimitiveField<i16, LittleEndian, 5>>;
        type Field2 = WrappedField<i16, Wrapped<i16>, PrimitiveField<i16, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(500));
        Field2::write(&mut storage, Wrapped(-2000));

        assert_eq!(
            500,
            i16::from_le_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            -2000,
            i16::from_le_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(Wrapped(500), Field1::read(&storage));
        assert_eq!(Wrapped(-2000), Field2::read(&storage));

        assert_eq!(
            Some(2),
            WrappedField::<i16, Wrapped<i16>, PrimitiveField::<i16, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(2),
            WrappedField::<i16, Wrapped<i16>, PrimitiveField::<i16, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i16_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i16, Wrapped<i16>, PrimitiveField<i16, BigEndian, 5>>;
        type Field2 = WrappedField<i16, Wrapped<i16>, PrimitiveField<i16, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(500));
        Field2::write(&mut storage, Wrapped(-2000));

        assert_eq!(
            500,
            i16::from_be_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            -2000,
            i16::from_be_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(Wrapped(500), Field1::read(&storage));
        assert_eq!(Wrapped(-2000), Field2::read(&storage));

        assert_eq!(
            Some(2),
            WrappedField::<i16, Wrapped<i16>, PrimitiveField::<i16, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(2),
            WrappedField::<i16, Wrapped<i16>, PrimitiveField::<i16, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i16_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i16, Wrapped<i16>, PrimitiveField<i16, NativeEndian, 5>>;
        type Field2 = WrappedField<i16, Wrapped<i16>, PrimitiveField<i16, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(500));
        Field2::write(&mut storage, Wrapped(-2000));

        assert_eq!(
            500,
            i16::from_ne_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            -2000,
            i16::from_ne_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(Wrapped(500), Field1::read(&storage));
        assert_eq!(Wrapped(-2000), Field2::read(&storage));

        assert_eq!(
            Some(2),
            WrappedField::<i16, Wrapped<i16>, PrimitiveField::<i16, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(2),
            WrappedField::<i16, Wrapped<i16>, PrimitiveField::<i16, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i32, Wrapped<i32>, PrimitiveField<i32, LittleEndian, 5>>;
        type Field2 = WrappedField<i32, Wrapped<i32>, PrimitiveField<i32, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10i32.pow(8)));
        Field2::write(&mut storage, Wrapped(-(10i32.pow(7))));

        assert_eq!(
            10i32.pow(8),
            i32::from_le_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            -(10i32.pow(7)),
            i32::from_le_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10i32.pow(8)), Field1::read(&storage));
        assert_eq!(Wrapped(-(10i32.pow(7))), Field2::read(&storage));

        assert_eq!(
            Some(4),
            WrappedField::<i32, Wrapped<i32>, PrimitiveField::<i32, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(4),
            WrappedField::<i32, Wrapped<i32>, PrimitiveField::<i32, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i32, Wrapped<i32>, PrimitiveField<i32, BigEndian, 5>>;
        type Field2 = WrappedField<i32, Wrapped<i32>, PrimitiveField<i32, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10i32.pow(8)));
        Field2::write(&mut storage, Wrapped(-(10i32.pow(7))));

        assert_eq!(
            10i32.pow(8),
            i32::from_be_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            -(10i32.pow(7)),
            i32::from_be_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10i32.pow(8)), Field1::read(&storage));
        assert_eq!(Wrapped(-(10i32.pow(7))), Field2::read(&storage));

        assert_eq!(
            Some(4),
            WrappedField::<i32, Wrapped<i32>, PrimitiveField::<i32, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(4),
            WrappedField::<i32, Wrapped<i32>, PrimitiveField::<i32, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i32_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i32, Wrapped<i32>, PrimitiveField<i32, NativeEndian, 5>>;
        type Field2 = WrappedField<i32, Wrapped<i32>, PrimitiveField<i32, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10i32.pow(8)));
        Field2::write(&mut storage, Wrapped(-(10i32.pow(7))));

        assert_eq!(
            10i32.pow(8),
            i32::from_ne_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            -(10i32.pow(7)),
            i32::from_ne_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10i32.pow(8)), Field1::read(&storage));
        assert_eq!(Wrapped(-(10i32.pow(7))), Field2::read(&storage));

        assert_eq!(
            Some(4),
            WrappedField::<i32, Wrapped<i32>, PrimitiveField::<i32, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(4),
            WrappedField::<i32, Wrapped<i32>, PrimitiveField::<i32, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i64, Wrapped<i64>, PrimitiveField<i64, LittleEndian, 5>>;
        type Field2 = WrappedField<i64, Wrapped<i64>, PrimitiveField<i64, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10i64.pow(15)));
        Field2::write(&mut storage, Wrapped(-(10i64.pow(14))));

        assert_eq!(
            10i64.pow(15),
            i64::from_le_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            -(10i64.pow(14)),
            i64::from_le_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10i64.pow(15)), Field1::read(&storage));
        assert_eq!(Wrapped(-(10i64.pow(14))), Field2::read(&storage));

        assert_eq!(
            Some(8),
            WrappedField::<i64, Wrapped<i64>, PrimitiveField::<i64, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(8),
            WrappedField::<i64, Wrapped<i64>, PrimitiveField::<i64, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i64, Wrapped<i64>, PrimitiveField<i64, BigEndian, 5>>;
        type Field2 = WrappedField<i64, Wrapped<i64>, PrimitiveField<i64, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10i64.pow(15)));
        Field2::write(&mut storage, Wrapped(-(10i64.pow(14))));

        assert_eq!(
            10i64.pow(15),
            i64::from_be_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            -(10i64.pow(14)),
            i64::from_be_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10i64.pow(15)), Field1::read(&storage));
        assert_eq!(Wrapped(-(10i64.pow(14))), Field2::read(&storage));

        assert_eq!(
            Some(8),
            WrappedField::<i64, Wrapped<i64>, PrimitiveField::<i64, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(8),
            WrappedField::<i64, Wrapped<i64>, PrimitiveField::<i64, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i64_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i64, Wrapped<i64>, PrimitiveField<i64, NativeEndian, 5>>;
        type Field2 = WrappedField<i64, Wrapped<i64>, PrimitiveField<i64, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10i64.pow(15)));
        Field2::write(&mut storage, Wrapped(-(10i64.pow(14))));

        assert_eq!(
            10i64.pow(15),
            i64::from_ne_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            -(10i64.pow(14)),
            i64::from_ne_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10i64.pow(15)), Field1::read(&storage));
        assert_eq!(Wrapped(-(10i64.pow(14))), Field2::read(&storage));

        assert_eq!(
            Some(8),
            WrappedField::<i64, Wrapped<i64>, PrimitiveField::<i64, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(8),
            WrappedField::<i64, Wrapped<i64>, PrimitiveField::<i64, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i128_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i128, Wrapped<i128>, PrimitiveField<i128, LittleEndian, 5>>;
        type Field2 = WrappedField<i128, Wrapped<i128>, PrimitiveField<i128, LittleEndian, 200>>;

        Field1::write(&mut storage, Wrapped(10i128.pow(30)));
        Field2::write(&mut storage, Wrapped(-(10i128.pow(28))));

        assert_eq!(
            10i128.pow(30),
            i128::from_le_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            -(10i128.pow(28)),
            i128::from_le_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10i128.pow(30)), Field1::read(&storage));
        assert_eq!(Wrapped(-(10i128.pow(28))), Field2::read(&storage));

        assert_eq!(
            Some(16),
            WrappedField::<i128, Wrapped<i128>, PrimitiveField::<i128, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(16),
            WrappedField::<i128, Wrapped<i128>, PrimitiveField::<i128, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i128_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i128, Wrapped<i128>, PrimitiveField<i128, BigEndian, 5>>;
        type Field2 = WrappedField<i128, Wrapped<i128>, PrimitiveField<i128, BigEndian, 200>>;

        Field1::write(&mut storage, Wrapped(10i128.pow(30)));
        Field2::write(&mut storage, Wrapped(-(10i128.pow(28))));

        assert_eq!(
            10i128.pow(30),
            i128::from_be_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            -(10i128.pow(28)),
            i128::from_be_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10i128.pow(30)), Field1::read(&storage));
        assert_eq!(Wrapped(-(10i128.pow(28))), Field2::read(&storage));

        assert_eq!(
            Some(16),
            WrappedField::<i128, Wrapped<i128>, PrimitiveField::<i128, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(16),
            WrappedField::<i128, Wrapped<i128>, PrimitiveField::<i128, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_i128_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<i128, Wrapped<i128>, PrimitiveField<i128, NativeEndian, 5>>;
        type Field2 = WrappedField<i128, Wrapped<i128>, PrimitiveField<i128, NativeEndian, 200>>;

        Field1::write(&mut storage, Wrapped(10i128.pow(30)));
        Field2::write(&mut storage, Wrapped(-(10i128.pow(28))));

        assert_eq!(
            10i128.pow(30),
            i128::from_ne_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            -(10i128.pow(28)),
            i128::from_ne_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10i128.pow(30)), Field1::read(&storage));
        assert_eq!(Wrapped(-(10i128.pow(28))), Field2::read(&storage));

        assert_eq!(
            Some(16),
            WrappedField::<i128, Wrapped<i128>, PrimitiveField::<i128, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(16),
            WrappedField::<i128, Wrapped<i128>, PrimitiveField::<i128, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u8_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u8, Wrapped<u8>, PrimitiveField<u8, LittleEndian, 5>>;
        type Field2 = WrappedField<u8, Wrapped<u8>, PrimitiveField<u8, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(50));
        Field2::write(&mut storage, Wrapped(20));

        assert_eq!(Wrapped(50), Field1::read(&storage));
        assert_eq!(Wrapped(20), Field2::read(&storage));

        assert_eq!(50, u8::from_le_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            20,
            u8::from_le_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(
            Some(1),
            WrappedField::<u8, Wrapped<u8>, PrimitiveField::<u8, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(1),
            WrappedField::<u8, Wrapped<u8>, PrimitiveField::<u8, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u8_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u8, Wrapped<u8>, PrimitiveField<u8, BigEndian, 5>>;
        type Field2 = WrappedField<u8, Wrapped<u8>, PrimitiveField<u8, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(50));
        Field2::write(&mut storage, Wrapped(20));

        assert_eq!(Wrapped(50), Field1::read(&storage));
        assert_eq!(Wrapped(20), Field2::read(&storage));

        assert_eq!(50, u8::from_be_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            20,
            u8::from_be_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(
            Some(1),
            WrappedField::<u8, Wrapped<u8>, PrimitiveField::<u8, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(1),
            WrappedField::<u8, Wrapped<u8>, PrimitiveField::<u8, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u8_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u8, Wrapped<u8>, PrimitiveField<u8, NativeEndian, 5>>;
        type Field2 = WrappedField<u8, Wrapped<u8>, PrimitiveField<u8, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(50));
        Field2::write(&mut storage, Wrapped(20));

        assert_eq!(Wrapped(50), Field1::read(&storage));
        assert_eq!(Wrapped(20), Field2::read(&storage));

        assert_eq!(50, u8::from_ne_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            20,
            u8::from_ne_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(
            Some(1),
            WrappedField::<u8, Wrapped<u8>, PrimitiveField::<u8, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(1),
            WrappedField::<u8, Wrapped<u8>, PrimitiveField::<u8, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u16_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u16, Wrapped<u16>, PrimitiveField<u16, LittleEndian, 5>>;
        type Field2 = WrappedField<u16, Wrapped<u16>, PrimitiveField<u16, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(500));
        Field2::write(&mut storage, Wrapped(2000));

        assert_eq!(
            500,
            u16::from_le_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            2000,
            u16::from_le_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(Wrapped(500), Field1::read(&storage));
        assert_eq!(Wrapped(2000), Field2::read(&storage));

        assert_eq!(
            Some(2),
            WrappedField::<u16, Wrapped<u16>, PrimitiveField::<u16, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(2),
            WrappedField::<u16, Wrapped<u16>, PrimitiveField::<u16, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u16_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u16, Wrapped<u16>, PrimitiveField<u16, BigEndian, 5>>;
        type Field2 = WrappedField<u16, Wrapped<u16>, PrimitiveField<u16, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(500));
        Field2::write(&mut storage, Wrapped(2000));

        assert_eq!(
            500,
            u16::from_be_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            2000,
            u16::from_be_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(Wrapped(500), Field1::read(&storage));
        assert_eq!(Wrapped(2000), Field2::read(&storage));

        assert_eq!(
            Some(2),
            WrappedField::<u16, Wrapped<u16>, PrimitiveField::<u16, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(2),
            WrappedField::<u16, Wrapped<u16>, PrimitiveField::<u16, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u16_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u16, Wrapped<u16>, PrimitiveField<u16, NativeEndian, 5>>;
        type Field2 = WrappedField<u16, Wrapped<u16>, PrimitiveField<u16, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(500));
        Field2::write(&mut storage, Wrapped(2000));

        assert_eq!(
            500,
            u16::from_ne_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            2000,
            u16::from_ne_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(Wrapped(500), Field1::read(&storage));
        assert_eq!(Wrapped(2000), Field2::read(&storage));

        assert_eq!(
            Some(2),
            WrappedField::<u16, Wrapped<u16>, PrimitiveField::<u16, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(2),
            WrappedField::<u16, Wrapped<u16>, PrimitiveField::<u16, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u32, Wrapped<u32>, PrimitiveField<u32, LittleEndian, 5>>;
        type Field2 = WrappedField<u32, Wrapped<u32>, PrimitiveField<u32, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10u32.pow(8)));
        Field2::write(&mut storage, Wrapped(10u32.pow(7)));

        assert_eq!(
            10u32.pow(8),
            u32::from_le_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10u32.pow(7),
            u32::from_le_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10u32.pow(8)), Field1::read(&storage));
        assert_eq!(Wrapped(10u32.pow(7)), Field2::read(&storage));

        assert_eq!(
            Some(4),
            WrappedField::<u32, Wrapped<u32>, PrimitiveField::<u32, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(4),
            WrappedField::<u32, Wrapped<u32>, PrimitiveField::<u32, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u32, Wrapped<u32>, PrimitiveField<u32, BigEndian, 5>>;
        type Field2 = WrappedField<u32, Wrapped<u32>, PrimitiveField<u32, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10u32.pow(8)));
        Field2::write(&mut storage, Wrapped(10u32.pow(7)));

        assert_eq!(
            10u32.pow(8),
            u32::from_be_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10u32.pow(7),
            u32::from_be_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10u32.pow(8)), Field1::read(&storage));
        assert_eq!(Wrapped(10u32.pow(7)), Field2::read(&storage));

        assert_eq!(
            Some(4),
            WrappedField::<u32, Wrapped<u32>, PrimitiveField::<u32, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(4),
            WrappedField::<u32, Wrapped<u32>, PrimitiveField::<u32, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u32_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u32, Wrapped<u32>, PrimitiveField<u32, NativeEndian, 5>>;
        type Field2 = WrappedField<u32, Wrapped<u32>, PrimitiveField<u32, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10u32.pow(8)));
        Field2::write(&mut storage, Wrapped(10u32.pow(7)));

        assert_eq!(
            10u32.pow(8),
            u32::from_ne_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10u32.pow(7),
            u32::from_ne_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10u32.pow(8)), Field1::read(&storage));
        assert_eq!(Wrapped(10u32.pow(7)), Field2::read(&storage));

        assert_eq!(
            Some(4),
            WrappedField::<u32, Wrapped<u32>, PrimitiveField::<u32, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(4),
            WrappedField::<u32, Wrapped<u32>, PrimitiveField::<u32, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u64, Wrapped<u64>, PrimitiveField<u64, LittleEndian, 5>>;
        type Field2 = WrappedField<u64, Wrapped<u64>, PrimitiveField<u64, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10u64.pow(15)));
        Field2::write(&mut storage, Wrapped(10u64.pow(14)));

        assert_eq!(
            10u64.pow(15),
            u64::from_le_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10u64.pow(14),
            u64::from_le_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10u64.pow(15)), Field1::read(&storage));
        assert_eq!(Wrapped(10u64.pow(14)), Field2::read(&storage));

        assert_eq!(
            Some(8),
            WrappedField::<u64, Wrapped<u64>, PrimitiveField::<u64, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(8),
            WrappedField::<u64, Wrapped<u64>, PrimitiveField::<u64, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u64, Wrapped<u64>, PrimitiveField<u64, BigEndian, 5>>;
        type Field2 = WrappedField<u64, Wrapped<u64>, PrimitiveField<u64, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10u64.pow(15)));
        Field2::write(&mut storage, Wrapped(10u64.pow(14)));

        assert_eq!(
            10u64.pow(15),
            u64::from_be_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10u64.pow(14),
            u64::from_be_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10u64.pow(15)), Field1::read(&storage));
        assert_eq!(Wrapped(10u64.pow(14)), Field2::read(&storage));

        assert_eq!(
            Some(8),
            WrappedField::<u64, Wrapped<u64>, PrimitiveField::<u64, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(8),
            WrappedField::<u64, Wrapped<u64>, PrimitiveField::<u64, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u64_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u64, Wrapped<u64>, PrimitiveField<u64, NativeEndian, 5>>;
        type Field2 = WrappedField<u64, Wrapped<u64>, PrimitiveField<u64, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10u64.pow(15)));
        Field2::write(&mut storage, Wrapped(10u64.pow(14)));

        assert_eq!(
            10u64.pow(15),
            u64::from_ne_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10u64.pow(14),
            u64::from_ne_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10u64.pow(15)), Field1::read(&storage));
        assert_eq!(Wrapped(10u64.pow(14)), Field2::read(&storage));

        assert_eq!(
            Some(8),
            WrappedField::<u64, Wrapped<u64>, PrimitiveField::<u64, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(8),
            WrappedField::<u64, Wrapped<u64>, PrimitiveField::<u64, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u128_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u128, Wrapped<u128>, PrimitiveField<u128, LittleEndian, 5>>;
        type Field2 = WrappedField<u128, Wrapped<u128>, PrimitiveField<u128, LittleEndian, 200>>;

        Field1::write(&mut storage, Wrapped(10u128.pow(30)));
        Field2::write(&mut storage, Wrapped(10u128.pow(28)));

        assert_eq!(
            10u128.pow(30),
            u128::from_le_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            10u128.pow(28),
            u128::from_le_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10u128.pow(30)), Field1::read(&storage));
        assert_eq!(Wrapped(10u128.pow(28)), Field2::read(&storage));

        assert_eq!(
            Some(16),
            WrappedField::<u128, Wrapped<u128>, PrimitiveField::<u128, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(16),
            WrappedField::<u128, Wrapped<u128>, PrimitiveField::<u128, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u128_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u128, Wrapped<u128>, PrimitiveField<u128, BigEndian, 5>>;
        type Field2 = WrappedField<u128, Wrapped<u128>, PrimitiveField<u128, BigEndian, 200>>;

        Field1::write(&mut storage, Wrapped(10u128.pow(30)));
        Field2::write(&mut storage, Wrapped(10u128.pow(28)));

        assert_eq!(
            10u128.pow(30),
            u128::from_be_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            10u128.pow(28),
            u128::from_be_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10u128.pow(30)), Field1::read(&storage));
        assert_eq!(Wrapped(10u128.pow(28)), Field2::read(&storage));

        assert_eq!(
            Some(16),
            WrappedField::<u128, Wrapped<u128>, PrimitiveField::<u128, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(16),
            WrappedField::<u128, Wrapped<u128>, PrimitiveField::<u128, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_u128_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<u128, Wrapped<u128>, PrimitiveField<u128, NativeEndian, 5>>;
        type Field2 = WrappedField<u128, Wrapped<u128>, PrimitiveField<u128, NativeEndian, 200>>;

        Field1::write(&mut storage, Wrapped(10u128.pow(30)));
        Field2::write(&mut storage, Wrapped(10u128.pow(28)));

        assert_eq!(
            10u128.pow(30),
            u128::from_ne_bytes((&storage[5..21]).try_into().unwrap())
        );
        assert_eq!(
            10u128.pow(28),
            u128::from_ne_bytes((&storage[200..216]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10u128.pow(30)), Field1::read(&storage));
        assert_eq!(Wrapped(10u128.pow(28)), Field2::read(&storage));

        assert_eq!(
            Some(16),
            WrappedField::<u128, Wrapped<u128>, PrimitiveField::<u128, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(16),
            WrappedField::<u128, Wrapped<u128>, PrimitiveField::<u128, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_f32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<f32, Wrapped<f32>, PrimitiveField<f32, LittleEndian, 5>>;
        type Field2 = WrappedField<f32, Wrapped<f32>, PrimitiveField<f32, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10f32.powf(8.31)));
        Field2::write(&mut storage, Wrapped(10f32.powf(7.31)));

        assert_eq!(
            10f32.powf(8.31),
            f32::from_le_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10f32.powf(7.31),
            f32::from_le_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10f32.powf(8.31)), Field1::read(&storage));
        assert_eq!(Wrapped(10f32.powf(7.31)), Field2::read(&storage));

        assert_eq!(
            Some(4),
            WrappedField::<f32, Wrapped<f32>, PrimitiveField::<f32, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(4),
            WrappedField::<f32, Wrapped<f32>, PrimitiveField::<f32, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_f32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<f32, Wrapped<f32>, PrimitiveField<f32, BigEndian, 5>>;
        type Field2 = WrappedField<f32, Wrapped<f32>, PrimitiveField<f32, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10f32.powf(8.31)));
        Field2::write(&mut storage, Wrapped(10f32.powf(7.31)));

        assert_eq!(
            10f32.powf(8.31),
            f32::from_be_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10f32.powf(7.31),
            f32::from_be_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10f32.powf(8.31)), Field1::read(&storage));
        assert_eq!(Wrapped(10f32.powf(7.31)), Field2::read(&storage));

        assert_eq!(
            Some(4),
            WrappedField::<f32, Wrapped<f32>, PrimitiveField::<f32, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(4),
            WrappedField::<f32, Wrapped<f32>, PrimitiveField::<f32, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_f32_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<f32, Wrapped<f32>, PrimitiveField<f32, NativeEndian, 5>>;
        type Field2 = WrappedField<f32, Wrapped<f32>, PrimitiveField<f32, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10f32.powf(8.31)));
        Field2::write(&mut storage, Wrapped(10f32.powf(7.31)));

        assert_eq!(
            10f32.powf(8.31),
            f32::from_ne_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10f32.powf(7.31),
            f32::from_ne_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10f32.powf(8.31)), Field1::read(&storage));
        assert_eq!(Wrapped(10f32.powf(7.31)), Field2::read(&storage));

        assert_eq!(
            Some(4),
            WrappedField::<f32, Wrapped<f32>, PrimitiveField::<f32, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(4),
            WrappedField::<f32, Wrapped<f32>, PrimitiveField::<f32, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_f64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<f64, Wrapped<f64>, PrimitiveField<f64, LittleEndian, 5>>;
        type Field2 = WrappedField<f64, Wrapped<f64>, PrimitiveField<f64, LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10f64.powf(15.31)));
        Field2::write(&mut storage, Wrapped(10f64.powf(14.31)));

        assert_eq!(
            10f64.powf(15.31),
            f64::from_le_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10f64.powf(14.31),
            f64::from_le_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10f64.powf(15.31)), Field1::read(&storage));
        assert_eq!(Wrapped(10f64.powf(14.31)), Field2::read(&storage));

        assert_eq!(
            Some(8),
            WrappedField::<f64, Wrapped<f64>, PrimitiveField::<f64, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(8),
            WrappedField::<f64, Wrapped<f64>, PrimitiveField::<f64, LittleEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_f64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<f64, Wrapped<f64>, PrimitiveField<f64, BigEndian, 5>>;
        type Field2 = WrappedField<f64, Wrapped<f64>, PrimitiveField<f64, BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10f64.powf(15.31)));
        Field2::write(&mut storage, Wrapped(10f64.powf(14.31)));

        assert_eq!(
            10f64.powf(15.31),
            f64::from_be_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10f64.powf(14.31),
            f64::from_be_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10f64.powf(15.31)), Field1::read(&storage));
        assert_eq!(Wrapped(10f64.powf(14.31)), Field2::read(&storage));

        assert_eq!(
            Some(8),
            WrappedField::<f64, Wrapped<f64>, PrimitiveField::<f64, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(8),
            WrappedField::<f64, Wrapped<f64>, PrimitiveField::<f64, BigEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_f64_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<f64, Wrapped<f64>, PrimitiveField<f64, NativeEndian, 5>>;
        type Field2 = WrappedField<f64, Wrapped<f64>, PrimitiveField<f64, NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(10f64.powf(15.31)));
        Field2::write(&mut storage, Wrapped(10f64.powf(14.31)));

        assert_eq!(
            10f64.powf(15.31),
            f64::from_ne_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10f64.powf(14.31),
            f64::from_ne_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(Wrapped(10f64.powf(15.31)), Field1::read(&storage));
        assert_eq!(Wrapped(10f64.powf(14.31)), Field2::read(&storage));

        assert_eq!(
            Some(8),
            WrappedField::<f64, Wrapped<f64>, PrimitiveField::<f64, NativeEndian, 5>>::SIZE
        );
        assert_eq!(
            Some(8),
            WrappedField::<f64, Wrapped<f64>, PrimitiveField::<f64, NativeEndian, 5>>::SIZE
        );
    }

    #[test]
    fn test_unit_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<(), Wrapped<()>, PrimitiveField<(), LittleEndian, 5>>;
        type Field2 = WrappedField<(), Wrapped<()>, PrimitiveField<(), LittleEndian, 20>>;

        Field1::write(&mut storage, Wrapped(()));
        Field2::write(&mut storage, Wrapped(()));

        assert_eq!(Wrapped(()), Field1::read(&storage));
        assert_eq!(Wrapped(()), Field2::read(&storage));

        // Unit is a zero sized type, so the storage should never be mutated.
        assert_eq!(storage, vec![0; 1024]);
    }

    #[test]
    fn test_unit_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<(), Wrapped<()>, PrimitiveField<(), BigEndian, 5>>;
        type Field2 = WrappedField<(), Wrapped<()>, PrimitiveField<(), BigEndian, 20>>;

        Field1::write(&mut storage, Wrapped(()));
        Field2::write(&mut storage, Wrapped(()));

        assert_eq!(Wrapped(()), Field1::read(&storage));
        assert_eq!(Wrapped(()), Field2::read(&storage));

        // Unit is a zero sized type, so the storage should never be mutated.
        assert_eq!(storage, vec![0; 1024]);
    }

    #[test]
    fn test_unit_nativeendian() {
        let mut storage = vec![0; 1024];

        type Field1 = WrappedField<(), Wrapped<()>, PrimitiveField<(), NativeEndian, 5>>;
        type Field2 = WrappedField<(), Wrapped<()>, PrimitiveField<(), NativeEndian, 20>>;

        Field1::write(&mut storage, Wrapped(()));
        Field2::write(&mut storage, Wrapped(()));

        assert_eq!(Wrapped(()), Field1::read(&storage));
        assert_eq!(Wrapped(()), Field2::read(&storage));

        // Unit is a zero sized type, so the storage should never be mutated.
        assert_eq!(storage, vec![0; 1024]);
    }
}
