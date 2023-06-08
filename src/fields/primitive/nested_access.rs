use super::super::{StorageIntoFieldView, StorageToFieldView};
use super::{Endianness, PrimitiveField};
use crate::utils::data::Data;
use crate::Field;

// Nesting generally works by having the define_layout! macro implement [OwningNestedView], [BorrowingNestedView]
// and [NestedViewInfo] for a marker type "NestedLayout" it creates in the layout's generated code.
// Then, the code in this module here creates implementations of [Field], [StorageToFieldView]
// and [StorageIntoFieldView] for it so that it can be used as a field in other layouts.

/// Internal type. Don't use this in user code.
/// S is expected to be a non-reference type that can own things, e.g. `Data<S>`
pub trait OwningNestedView<S>
where
    S: AsRef<[u8]>,
{
    /// A type representing an owning view of the nested field.
    type View;

    /// Takes a storage pointing only to the space of the subfield and returns a view to the subfield, with the view taking ownership of the storage.
    fn into_view(storage: S) -> Self::View;
}

/// Internal type. Don't use this in user code.
/// S is expected to be a reference type, e.g. &[u8] or &mut [u8]
pub trait BorrowingNestedView<S> {
    /// A type representing a borrowing view of the nested field.
    type View;

    /// Takes a storage pointing only to the space of the subfield and returns a view to the subfield, with the view not taking ownership of the storage.
    fn view(storage: S) -> Self::View;
}

/// Internal trait. Don't use this in user code.
pub trait NestedViewInfo {
    /// Size of the nested field
    const SIZE: Option<usize>;
}

// TODO FieldNestedAccess may be useful for the field API, but commented out for now since the field API doesn't support nesting yet
// /// This trait is implemented for fields with "nested access",
// /// i.e. fields that represent other layouts that are nested within
// /// this layout.
// pub trait FieldNestedAccess<'a, S>: Field {
//     /// A view type for the nested field that owns its storage
//     type OwningView;

//     /// A view type for the nested field that immutably borrows its storage
//     type BorrowedView;

//     /// A view type for the nested field that mutably borrows its storage
//     type BorrowedViewMut;

//     fn into_view(storage: Data<S>) -> Self::OwningView;
//     fn view(storage: &'a [u8]) -> Self::BorrowedView;
//     fn view_mut(storage: &'a mut [u8]) -> Self::BorrowedViewMut;
// }

// impl<'a, S, T, E, const OFFSET_: usize> FieldNestedAccess<'a, S> for PrimitiveField<T, E, OFFSET_>
// where
//     S: AsRef<[u8]>,
//     T: OwningNestedView<Data<S>>
//         + BorrowingNestedView<&'a [u8]>
//         + BorrowingNestedView<&'a mut [u8]>,
//     E: Endianness,
//     Self: Field,
// {
//     type OwningView = <T as OwningNestedView<Data<S>>>::View;
//     type BorrowedView = <T as BorrowingNestedView<&'a [u8]>>::View;
//     type BorrowedViewMut = <T as BorrowingNestedView<&'a mut [u8]>>::View;

//     #[inline(always)]
//     fn into_view(storage: Data<S>) -> Self::OwningView {
//         let data = if let Some(size) = Self::SIZE {
//             Data::from(storage).into_subregion(Self::OFFSET..(Self::OFFSET + size))
//         } else {
//             Data::from(storage).into_subregion(Self::OFFSET..)
//         };
//         T::into_view(data)
//     }

//     #[inline(always)]
//     fn view(storage: &'a [u8]) -> Self::BorrowedView {
//         let data = if let Some(size) = Self::SIZE {
//             &storage[Self::OFFSET..(Self::OFFSET + size)]
//         } else {
//             &storage[Self::OFFSET..]
//         };
//         T::view(data)
//     }

//     #[inline(always)]
//     fn view_mut(storage: &'a mut [u8]) -> Self::BorrowedViewMut {
//         let data = if let Some(size) = Self::SIZE {
//             &mut storage[Self::OFFSET..(Self::OFFSET + size)]
//         } else {
//             &mut storage[Self::OFFSET..]
//         };
//         T::view(data)
//     }
// }

impl<N: NestedViewInfo, E: Endianness, const OFFSET_: usize> Field
    for PrimitiveField<N, E, OFFSET_>
{
    /// See [Field::Endian]
    type Endian = E;
    /// See [Field::OFFSET]
    const OFFSET: usize = OFFSET_;
    /// See [Field::SIZE]
    const SIZE: Option<usize> = N::SIZE;
}

impl<'a, N: BorrowingNestedView<&'a [u8]>, E: Endianness, const OFFSET_: usize>
    StorageToFieldView<&'a [u8]> for PrimitiveField<N, E, OFFSET_>
where
    Self: Field,
{
    type View = N::View;

    #[inline(always)]
    fn view(storage: &'a [u8]) -> Self::View {
        if let Some(size) = Self::SIZE {
            N::view(&storage[Self::OFFSET..(Self::OFFSET + size)])
        } else {
            N::view(&storage[Self::OFFSET..])
        }
    }
}
impl<'a, N: BorrowingNestedView<&'a mut [u8]>, E: Endianness, const OFFSET_: usize>
    StorageToFieldView<&'a mut [u8]> for PrimitiveField<N, E, OFFSET_>
where
    Self: Field,
{
    type View = N::View;

    #[inline(always)]
    fn view(storage: &'a mut [u8]) -> Self::View {
        if let Some(size) = Self::SIZE {
            N::view(&mut storage[Self::OFFSET..(Self::OFFSET + size)])
        } else {
            N::view(&mut storage[Self::OFFSET..])
        }
    }
}

impl<S: AsRef<[u8]>, N: OwningNestedView<Data<S>>, E: Endianness, const OFFSET_: usize>
    StorageIntoFieldView<S> for PrimitiveField<N, E, OFFSET_>
where
    Self: Field,
{
    type View = N::View;

    #[inline(always)]
    fn into_view(storage: S) -> Self::View {
        if let Some(size) = Self::SIZE {
            N::into_view(Data::from(storage).into_subregion(Self::OFFSET..(Self::OFFSET + size)))
        } else {
            N::into_view(Data::from(storage).into_subregion(Self::OFFSET..))
        }
    }
}
