// use core::convert::Infallible;

// use super::{FieldCopyAccess, PrimitiveField};
// use crate::endianness::Endianness;
// use crate::fields::primitive::view::FieldView;
// use crate::fields::{Field, StorageIntoFieldView, StorageToFieldView};

// impl<E: Endianness, const OFFSET_: usize> FieldCopyAccess for PrimitiveField<(), E, OFFSET_> {
//     /// See [FieldCopyAccess::ReadError]
//     type ReadError = Infallible;
//     /// See [FieldCopyAccess::WriteError]
//     type WriteError = Infallible;
//     /// See [FieldCopyAccess::HighLevelType]
//     type HighLevelType = ();

//     doc_comment::doc_comment! {
//         concat! {"
//                 'Read' the `", stringify!(()), "`-typed field from a given data region, assuming the defined layout, using the [Field] API.

//                 # Example:

//                 ```
//                 use binary_layout::prelude::*;

//                 binary_layout!(my_layout, LittleEndian, {
//                     //... other fields ...
//                     some_zst_field: ", stringify!(()), "
//                     //... other fields ...
//                 });

//                 fn func(storage_data: &[u8]) {
//                     let read: ", stringify!(()), " = my_layout::some_zst_field::try_read(storage_data).unwrap();
//                     read
//                 }
//                 ```

//                 In reality, this method doesn't do any work; `",
//                 stringify!(()), "` is a zero-sized type, so there's no work to
//                 do. This implementation exists solely to make writing derive
//                 macros simpler.
//                 "},
//         #[inline(always)]
//         #[allow(clippy::unused_unit)] // I don't want to remove this as it's part of the trait.
//         fn try_read(_storage: &[u8]) -> Result<(), Infallible> {
//             Ok(())
//         }
//     }

//     doc_comment::doc_comment! {
//         concat! {"
//                 'Write' the `", stringify!(()), "`-typed field to a given data region, assuming the defined layout, using the [Field] API.

//                 # Example:

//                 ```
//                 use binary_layout::prelude::*;

//                 binary_layout!(my_layout, LittleEndian, {
//                     //... other fields ...
//                     some_zst_field: ", stringify!(()), "
//                     //... other fields ...
//                 });

//                 fn func(storage_data: &mut [u8]) {
//                     my_layout::some_zst_field::try_write(storage_data, ()).unwrap();
//                 }
//                 ```

//                 # WARNING

//                 In reality, this method doesn't do any work; `",
//                 stringify!(()), "` is a zero-sized type, so there's no work to
//                 do. This implementation exists solely to make writing derive
//                 macros simpler.
//                 "},
//         #[inline(always)]
//         #[allow(clippy::unused_unit)] // I don't want to remove this as it's part of the trait.
//         fn try_write(_storage: &mut [u8], _value: ()) -> Result<(), Infallible> {
//             Ok(())
//         }
//     }
// }

// impl_field_traits!(());

// #[cfg(test)]
// mod tests {
//     use crate::prelude::*;
//     use crate::PrimitiveField;

//     macro_rules! test_unit_copy_access {
//         ($endian:ident, $endian_type:ty) => {
//             $crate::internal::paste! {
//                 #[test]
//                 fn [<test_unit_ $endian endian_metadata>]() {
//                     type Field1 = PrimitiveField<(), $endian_type, 5>;
//                     type Field2 = PrimitiveField<(), $endian_type, 123>;

//                     assert_eq!(Some(0), Field1::SIZE);
//                     assert_eq!(5, Field1::OFFSET);
//                     assert_eq!(Some(0), Field2::SIZE);
//                     assert_eq!(123, Field2::OFFSET);
//                 }

//                 #[allow(clippy::unit_cmp)]
//                 #[test]
//                 fn [<test_unit_ $endian endian_fieldapi>]() {
//                     let mut storage = [0; 1024];

//                     type Field1 = PrimitiveField<(), $endian_type, 5>;
//                     type Field2 = PrimitiveField<(), $endian_type, 123>;

//                     Field1::write(&mut storage, ());
//                     Field2::write(&mut storage, ());

//                     assert_eq!((), Field1::read(&storage));
//                     assert_eq!((), Field2::read(&storage));

//                     // Zero-sized types do not mutate the storage, so it should remain
//                     // unchanged for all of time.
//                     assert_eq!(storage, [0; 1024]);
//                 }

//                 #[allow(clippy::unit_cmp)]
//                 #[test]
//                 fn [<test_unit_ $endian endian_viewapi>]() {
//                     binary_layout!(layout, $endian_type, {
//                         field1: (),
//                         field2: (),
//                     });
//                     let mut storage = [0; 1024];
//                     let mut view = layout::View::new(&mut storage);

//                     view.field1_mut().write(());
//                     view.field2_mut().write(());

//                     assert_eq!((), view.field1().read());
//                     assert_eq!((), view.field2().read());

//                     // Zero-sized types do not mutate the storage, so it should remain
//                     // unchanged for all of time.
//                     assert_eq!(storage, [0; 1024]);
//                 }
//             }
//         };
//     }

//     test_unit_copy_access!(little, LittleEndian);
//     test_unit_copy_access!(big, BigEndian);
//     test_unit_copy_access!(native, NativeEndian);
// }
