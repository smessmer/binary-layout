use super::{PrimitiveRead, PrimitiveWrite};
use crate::data_types::DataTypeMetadata;
use crate::endianness::Endianness;
use crate::view::PrimitiveFieldView;
use crate::Field;

impl DataTypeMetadata for () {
    const SIZE: Option<usize> = Some(0);

    type View<S, F> = PrimitiveFieldView<S, F> where F: Field;
}

impl PrimitiveRead for () {
    /// Reading unit values can't fail
    type Error = core::convert::Infallible;

    /// Read the unit field from a given storage.
    /// The storage slice size must be of size zero, otherwise this will panic.
    #[inline(always)]
    fn try_read<E: Endianness>(storage: &[u8]) -> Result<Self, Self::Error> {
        assert_eq!(0, storage.len());
        Ok(())
    }
}

impl PrimitiveWrite for () {
    /// Writing unit values can't fail
    type Error = core::convert::Infallible;

    /// Write the unit field to a given storage.
    /// The storage slice size must be of size zero, otherwise this will panic.
    #[inline(always)]
    fn try_write<E: Endianness>(self, storage: &mut [u8]) -> Result<(), Self::Error> {
        assert_eq!(0, storage.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::PrimitiveField;

    macro_rules! test_unit_copy_access {
        ($endian:ident, $endian_type:ty) => {
            $crate::internal::paste! {
                #[test]
                fn [<test_unit_ $endian endian_metadata>]() {
                    type Field1 = PrimitiveField<(), $endian_type, 5>;
                    type Field2 = PrimitiveField<(), $endian_type, 123>;

                    assert_eq!(Some(0), Field1::SIZE);
                    assert_eq!(5, Field1::OFFSET);
                    assert_eq!(Some(0), Field2::SIZE);
                    assert_eq!(123, Field2::OFFSET);
                }

                #[allow(clippy::unit_cmp)]
                #[test]
                fn [<test_unit_ $endian endian_fieldapi>]() {
                    let mut storage = [0; 1024];

                    type Field1 = PrimitiveField<(), $endian_type, 5>;
                    type Field2 = PrimitiveField<(), $endian_type, 123>;

                    Field1::write(&mut storage, ());
                    Field2::write(&mut storage, ());

                    assert_eq!((), Field1::read(&storage));
                    assert_eq!((), Field2::read(&storage));

                    // Zero-sized types do not mutate the storage, so it should remain
                    // unchanged for all of time.
                    assert_eq!(storage, [0; 1024]);
                }

                #[allow(clippy::unit_cmp)]
                #[test]
                fn [<test_unit_ $endian endian_viewapi>]() {
                    binary_layout!(layout, $endian_type, {
                        field1: (),
                        field2: (),
                    });
                    let mut storage = [0; 1024];
                    let mut view = layout::View::new(&mut storage);

                    view.field1_mut().write(());
                    view.field2_mut().write(());

                    assert_eq!((), view.field1().read());
                    assert_eq!((), view.field2().read());

                    // Zero-sized types do not mutate the storage, so it should remain
                    // unchanged for all of time.
                    assert_eq!(storage, [0; 1024]);
                }
            }
        };
    }

    test_unit_copy_access!(little, LittleEndian);
    test_unit_copy_access!(big, BigEndian);
    test_unit_copy_access!(native, NativeEndian);
}
