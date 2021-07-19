use std::marker::PhantomData;

use super::{IField, IFieldCopyAccess, ISizedField};

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

/// TODO Doc
pub struct WrappedField<U, T, F: IField> {
    _p1: PhantomData<U>,
    _p2: PhantomData<T>,
    _p3: PhantomData<F>,
}

impl<U, T, F: IField> IField for WrappedField<U, T, F> {
    // TODO Doc
    type Endian = F::Endian;
    // TODO Doc
    const OFFSET: usize = F::OFFSET;
}

impl<U, T, F: ISizedField> ISizedField for WrappedField<U, T, F> {
    // TODO Doc
    const SIZE: usize = F::SIZE;
}

impl<U, T: LayoutAs<U>, F: IFieldCopyAccess<HighLevelType = U>> IFieldCopyAccess
    for WrappedField<U, T, F>
{
    /// TODO Doc
    type HighLevelType = T;

    /// TODO Doc
    fn read(storage: &[u8]) -> Self::HighLevelType {
        let v = F::read(storage);
        <T as LayoutAs<U>>::read(v)
    }

    /// TODO Doc
    fn write(storage: &mut [u8], v: Self::HighLevelType) {
        let v = <T as LayoutAs<U>>::write(v);
        F::write(storage, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endianness::{BigEndian, LittleEndian};
    use crate::PrimitiveField;
    use std::convert::TryInto;

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
            1,
            WrappedField::<i8, Wrapped<i8>, PrimitiveField::<i8, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            1,
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
            1,
            WrappedField::<i8, Wrapped<i8>, PrimitiveField::<i8, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            1,
            WrappedField::<i8, Wrapped<i8>, PrimitiveField::<i8, BigEndian, 5>>::SIZE
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
            2,
            WrappedField::<i16, Wrapped<i16>, PrimitiveField::<i16, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            2,
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
            2,
            WrappedField::<i16, Wrapped<i16>, PrimitiveField::<i16, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            2,
            WrappedField::<i16, Wrapped<i16>, PrimitiveField::<i16, BigEndian, 5>>::SIZE
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
            4,
            WrappedField::<i32, Wrapped<i32>, PrimitiveField::<i32, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            4,
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
            4,
            WrappedField::<i32, Wrapped<i32>, PrimitiveField::<i32, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            4,
            WrappedField::<i32, Wrapped<i32>, PrimitiveField::<i32, BigEndian, 5>>::SIZE
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
            8,
            WrappedField::<i64, Wrapped<i64>, PrimitiveField::<i64, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            8,
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
            8,
            WrappedField::<i64, Wrapped<i64>, PrimitiveField::<i64, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            8,
            WrappedField::<i64, Wrapped<i64>, PrimitiveField::<i64, BigEndian, 5>>::SIZE
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
            1,
            WrappedField::<u8, Wrapped<u8>, PrimitiveField::<u8, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            1,
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
            1,
            WrappedField::<u8, Wrapped<u8>, PrimitiveField::<u8, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            1,
            WrappedField::<u8, Wrapped<u8>, PrimitiveField::<u8, BigEndian, 5>>::SIZE
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
            2,
            WrappedField::<u16, Wrapped<u16>, PrimitiveField::<u16, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            2,
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
            2,
            WrappedField::<u16, Wrapped<u16>, PrimitiveField::<u16, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            2,
            WrappedField::<u16, Wrapped<u16>, PrimitiveField::<u16, BigEndian, 5>>::SIZE
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
            4,
            WrappedField::<u32, Wrapped<u32>, PrimitiveField::<u32, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            4,
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
            4,
            WrappedField::<u32, Wrapped<u32>, PrimitiveField::<u32, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            4,
            WrappedField::<u32, Wrapped<u32>, PrimitiveField::<u32, BigEndian, 5>>::SIZE
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
            8,
            WrappedField::<u64, Wrapped<u64>, PrimitiveField::<u64, LittleEndian, 5>>::SIZE
        );
        assert_eq!(
            8,
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
            8,
            WrappedField::<u64, Wrapped<u64>, PrimitiveField::<u64, BigEndian, 5>>::SIZE
        );
        assert_eq!(
            8,
            WrappedField::<u64, Wrapped<u64>, PrimitiveField::<u64, BigEndian, 5>>::SIZE
        );
    }
}
