use super::fields::{Endianness, Field, FieldSize, PrimitiveField};

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
/// impl LayoutAs for MyIdType {
///   type Underlying = u64;
///
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
///   field: MyIdType,
///   // ... other fields ...
/// });
/// ```
pub trait LayoutAs {
    /// The binary representation used to layout this type.
    /// This must be one of the [supported field types](crate#supported-field-types).
    type Underlying;

    /// Implement this to define how the custom type is constructed from the underlying type
    /// after it was read from a layouted binary slice.
    fn read(v: Self::Underlying) -> Self;

    /// Implement this to define how the custom type is converted into the underlying type
    /// so it can be written into a layouted binary slice.
    fn write(v: Self) -> Self::Underlying;
}

impl<T, U, E: Endianness, const OFFSET_: usize> PrimitiveField for Field<T, E, OFFSET_>
where
    T: LayoutAs<Underlying = U>,
    Field<U, E, OFFSET_>: PrimitiveField<FieldType = U>,
{
    type FieldType = T;

    fn read(storage: &[u8]) -> Self::FieldType {
        let underlying: <T as LayoutAs>::Underlying =
            <Field<<T as LayoutAs>::Underlying, E, OFFSET_> as PrimitiveField>::read(storage);
        <T as LayoutAs>::read(underlying)
    }

    fn write(storage: &mut [u8], value: Self::FieldType) {
        let underlying = <T as LayoutAs>::write(value);
        <Field<<T as LayoutAs>::Underlying, E, OFFSET_> as PrimitiveField>::write(
            storage, underlying,
        );
    }
}

impl<T, U> FieldSize for T
where
    T: LayoutAs<Underlying = U>,
    U: FieldSize,
{
    const SIZE: usize = U::SIZE;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::define_layout;
    use crate::testutils::data_region;
    use crate::{FieldMetadata, SizedFieldMetadata};
    use std::convert::TryInto;

    #[derive(Debug, PartialEq, Eq)]
    pub struct Wrapper<T>(T);
    impl<T> LayoutAs for Wrapper<T> {
        type Underlying = T;
        fn read(v: T) -> Wrapper<T> {
            Wrapper(v)
        }
        fn write(v: Wrapper<T>) -> T {
            v.0
        }
    }

    define_layout!(layout, LittleEndian, {
        first: Wrapper<i8>,
        second: Wrapper<i64>,
        third: Wrapper<u16>,
    });

    #[test]
    fn metadata() {
        assert_eq!(0, layout::first::OFFSET);
        assert_eq!(1, layout::first::SIZE);
        assert_eq!(1, layout::second::OFFSET);
        assert_eq!(8, layout::second::SIZE);
        assert_eq!(9, layout::third::OFFSET);
        assert_eq!(2, layout::third::SIZE);
    }

    #[test]
    fn fields() {
        let mut storage = data_region(1024, 5);

        // Test initial data is read correctly
        assert_eq!(
            Wrapper(i8::from_le_bytes(
                (&data_region(1024, 5)[0..1]).try_into().unwrap()
            )),
            layout::first::read(&storage)
        );
        assert_eq!(
            Wrapper(i64::from_le_bytes(
                (&data_region(1024, 5)[1..9]).try_into().unwrap()
            )),
            layout::second::read(&storage)
        );
        assert_eq!(
            Wrapper(u16::from_le_bytes(
                (&data_region(1024, 5)[9..11]).try_into().unwrap()
            )),
            layout::third::read(&storage)
        );

        // Test data can be written
        layout::first::write(&mut storage, Wrapper(60));
        layout::second::write(&mut storage, Wrapper(-100_000_000_000));
        layout::third::write(&mut storage, Wrapper(1_000));

        // Test reading will return changed data
        assert_eq!(60, layout::first::read(&storage).0);
        assert_eq!(-100_000_000_000, layout::second::read(&storage).0);
        assert_eq!(1_000, layout::third::read(&storage).0);
    }

    #[test]
    fn view_readonly() {
        let storage = data_region(1024, 5);
        let view = layout::View::new(&storage);

        // Test initial data is read correctly
        assert_eq!(
            Wrapper(i8::from_le_bytes(
                (&data_region(1024, 5)[0..1]).try_into().unwrap()
            )),
            view.first().read()
        );
        assert_eq!(
            Wrapper(i64::from_le_bytes(
                (&data_region(1024, 5)[1..9]).try_into().unwrap()
            )),
            view.second().read()
        );
        assert_eq!(
            Wrapper(u16::from_le_bytes(
                (&data_region(1024, 5)[9..11]).try_into().unwrap()
            )),
            view.third().read()
        );

        // Test into_storage will return correct data
        let extracted_storage = view.into_storage();
        assert_eq!(extracted_storage, &storage);
    }

    #[test]
    fn view_readwrite() {
        let mut storage = data_region(1024, 5);
        let mut view = layout::View::new(&mut storage);

        // Test initial data is read correctly
        assert_eq!(
            Wrapper(i8::from_le_bytes(
                (&data_region(1024, 5)[0..1]).try_into().unwrap()
            )),
            view.first().read()
        );
        assert_eq!(
            Wrapper(i64::from_le_bytes(
                (&data_region(1024, 5)[1..9]).try_into().unwrap()
            )),
            view.second().read()
        );
        assert_eq!(
            Wrapper(u16::from_le_bytes(
                (&data_region(1024, 5)[9..11]).try_into().unwrap()
            )),
            view.third().read()
        );

        // Test data can be written
        view.first_mut().write(Wrapper(50));
        view.second_mut().write(Wrapper(10i64.pow(15)));
        view.third_mut().write(Wrapper(1000));

        // Test reading will return changed data
        assert_eq!(Wrapper(50), view.first().read());
        assert_eq!(Wrapper(10i64.pow(15)), view.second().read());
        assert_eq!(Wrapper(1000), view.third().read());

        // Test into_storage will return correct data
        let extracted_storage = view.into_storage().clone();
        assert_eq!(&storage, &extracted_storage);

        // Test original storage is actually changed
        assert_eq!(50, i8::from_le_bytes((&storage[0..1]).try_into().unwrap()));
        assert_eq!(
            10i64.pow(15),
            i64::from_le_bytes((&storage[1..9]).try_into().unwrap())
        );
        assert_eq!(
            1000,
            u16::from_le_bytes((&storage[9..11]).try_into().unwrap())
        );
    }

    #[test]
    fn view_vec_readonly() {
        let view = layout::View::new(data_region(1024, 5));

        // Test initial data is read correctly
        assert_eq!(
            Wrapper(i8::from_le_bytes(
                (&data_region(1024, 5)[0..1]).try_into().unwrap()
            )),
            view.first().read()
        );
        assert_eq!(
            Wrapper(i64::from_le_bytes(
                (&data_region(1024, 5)[1..9]).try_into().unwrap()
            )),
            view.second().read()
        );
        assert_eq!(
            Wrapper(u16::from_le_bytes(
                (&data_region(1024, 5)[9..11]).try_into().unwrap()
            )),
            view.third().read()
        );

        // Test into_storage will return correct data
        let extracted_storage = view.into_storage();
        assert_eq!(&data_region(1024, 5), &extracted_storage);
    }

    #[test]
    fn view_vec_readwrite() {
        let mut view = layout::View::new(data_region(1024, 5));

        // Test initial data is read correctly
        assert_eq!(
            Wrapper(i8::from_le_bytes(
                (&data_region(1024, 5)[0..1]).try_into().unwrap()
            )),
            view.first().read()
        );
        assert_eq!(
            Wrapper(i64::from_le_bytes(
                (&data_region(1024, 5)[1..9]).try_into().unwrap()
            )),
            view.second().read()
        );
        assert_eq!(
            Wrapper(u16::from_le_bytes(
                (&data_region(1024, 5)[9..11]).try_into().unwrap()
            )),
            view.third().read()
        );

        // Test data can be written
        view.first_mut().write(Wrapper(50));
        view.second_mut().write(Wrapper(10i64.pow(15)));
        view.third_mut().write(Wrapper(1000));

        // Test reading will return changed data
        assert_eq!(Wrapper(50), view.first().read());
        assert_eq!(Wrapper(10i64.pow(15)), view.second().read());
        assert_eq!(Wrapper(1000), view.third().read());

        // Test into_storage will return correct data
        let extracted_storage = view.into_storage();
        assert_eq!(
            50,
            i8::from_le_bytes((&extracted_storage[0..1]).try_into().unwrap())
        );
        assert_eq!(
            10i64.pow(15),
            i64::from_le_bytes((&extracted_storage[1..9]).try_into().unwrap())
        );
        assert_eq!(
            1000,
            u16::from_le_bytes((&extracted_storage[9..11]).try_into().unwrap())
        );
    }
}
