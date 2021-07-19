use std::convert::TryFrom;
use std::marker::PhantomData;

// TODO Have implementations of trait functions inherit docs from trait instead of copy&pasting them, is there a crate for this?

/// This is an internal type that only exists because [const_evaluatable_checked](https://github.com/rust-lang/rust/issues/76560) isn't stabilized yet.
/// TODO Once [const_evaluatable_checked](https://github.com/rust-lang/rust/issues/76560) is stabilized, we should delete this and
/// instead add a SIZE constant to [Field].
// pub trait FieldSize {
//     /// Returns the size the data type would occupy in a layout.
//     const SIZE: usize;
// }

/// An enum representing the endianness used in a layout for accessing primitive integer fields.
pub enum EndianKind {
    Big,
    Little,
}

/// This marker trait represents the endianness used in a layout for accessing primitive integer fields.
pub trait Endianness {
    const KIND: EndianKind;
}

/// This is a marker type to mark layouts using big endian encoding. The alternative is [LittleEndian] encoding.
///
/// # Example
/// ```
/// use binary_layout::prelude::*;
///
/// define_layout!(my_layout, BigEndian, {
///   field1: i16,
///   field2: u32,
/// });
/// ```
pub struct BigEndian {}
impl Endianness for BigEndian {
    const KIND: EndianKind = EndianKind::Big;
}

/// This is a marker type to mark layouts using little endian encoding. The alternative is [BigEndian] encoding.
///
/// # Example
/// ```
/// use binary_layout::prelude::*;
///
/// define_layout!(my_layout, LittleEndian, {
///   field1: i16,
///   field2: u32,
/// });
/// ```
pub struct LittleEndian {}
impl Endianness for LittleEndian {
    const KIND: EndianKind = EndianKind::Little;
}

/// This trait offers access to the metadata of a field in a layout
pub trait IField {
    /// The endianness of the field
    type Endian: Endianness;

    /// The offset of the field in the layout.
    ///
    /// # Example
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   field1: u16,
    ///   field2: i32,
    ///   field3: u8,
    /// });
    ///
    /// assert_eq!(0, my_layout::field1::OFFSET);
    /// assert_eq!(2, my_layout::field2::OFFSET);
    /// assert_eq!(6, my_layout::field3::OFFSET);
    /// ```
    const OFFSET: usize;
}

/// This trait offers access to the metadata of a sized field in a layout.
/// Sized fields are all fields with a defined size. This is almost all fields.
/// The only exception is an unsized array field that can be used to match
/// tail data, i.e. any data at the end of the storage after all other fields
/// were defined and until the storage ends.
pub trait ISizedField: IField {
    /// The size of the field in the layout.
    ///
    /// # Example
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///   field1: u16,
    ///   field2: i32,
    ///   field3: u8,
    /// });
    ///
    /// assert_eq!(2, my_layout::field1::SIZE);
    /// assert_eq!(4, my_layout::field2::SIZE);
    /// assert_eq!(1, my_layout::field3::SIZE);
    /// ```
    const SIZE: usize;
}

/// This trait is implemented for fields with "copy access",
/// i.e. fields that read/write data by copying it from the
/// binary blob. Examples of this are primitive types
/// like u8, i32, ...
pub trait IFieldCopyAccess: IField {
    /// The data type that is returned from read calls and has to be
    /// passed in to write calls. This can be different from the primitive
    /// type used in the binary blob, since that primitive type can be
    /// wrapped into a high level type before being returned from read
    /// calls (or vice versa unwrapped when writing).
    type HighLevelType;

    /// TODO Doc
    fn read(storage: &[u8]) -> Self::HighLevelType;
    /// TODO Doc
    fn write(storage: &mut [u8], v: Self::HighLevelType);
}

/// This trait si implemented for fields with "slice access",
/// i.e. fields that are read/write directly without a copy
/// by returning a borrowed slice to the underlying data.
pub trait IFieldSliceAccess<'a>: IField {
    /// TODO Doc
    type SliceType: 'a;
    /// TODO Doc
    type MutSliceType: 'a;

    /// TODO Doc
    fn data(storage: &'a [u8]) -> Self::SliceType;
    /// TODO Doc
    fn data_mut(storage: &'a mut [u8]) -> Self::MutSliceType;
}

/// A field represents one of the fields in the data layout and offers accessors
/// for it. It remembers the offset of the field in its const generic parameter
/// and the accessors use that to access the field.
///
/// A field does not hold any data storage, so if you use this API directly, you have to pass in
/// the storage pointer for each call. If you want an API object that remembers the storage,
/// take a look at the [FieldView] based API instead.
///
/// # Example:
/// ```
/// use binary_layout::prelude::*;
///
/// define_layout!(my_layout, LittleEndian, {
///   field_one: u16,
///   another_field: [u8; 16],
///   something_else: u32,
///   tail_data: [u8],
/// });
///
/// fn func(storage_data: &mut [u8]) {
///   // read some data
///   let format_version_header: u16 = my_layout::field_one::read(storage_data);
///   // equivalent: let format_version_header = u16::from_le_bytes((&storage_data[0..2]).try_into().unwrap());
///
///   // write some data
///   my_layout::something_else::write(storage_data, 10);
///   // equivalent: data_slice[18..22].copy_from_slice(&10u32.to_le_bytes());
///
///   // access a data region
///   let tail_data: &[u8] = my_layout::tail_data::data(storage_data);
///   // equivalent: let tail_data: &[u8] = &data_slice[22..];
///
///   // and modify it
///   my_layout::tail_data::data_mut(storage_data)[..5].copy_from_slice(&[1, 2, 3, 4, 5]);
///   // equivalent: data_slice[18..22].copy_from_slice(&[1, 2, 3, 4, 5]);
/// }
/// ```
pub struct PrimitiveField<T: ?Sized, E: Endianness, const OFFSET_: usize> {
    _p1: PhantomData<T>,
    _p2: PhantomData<E>,
}

impl<T: ?Sized, E: Endianness, const OFFSET_: usize> IField for PrimitiveField<T, E, OFFSET_> {
    // TODO Doc
    type Endian = E;
    // TODO Doc
    const OFFSET: usize = OFFSET_;
}

/// TODO Doc
pub struct Field<U: IField> {
    _p1: PhantomData<U>,
}

impl<U: IField> IField for Field<U> {
    // TODO Doc
    type Endian = U::Endian;
    // TODO Doc
    const OFFSET: usize = U::OFFSET;
}

/// A field view represents the field metadata stored in a [Field] plus it stores the underlying
/// storage data it operates on, either as a reference to a slice `&[u8]`, `&mut [u8]`, or as
/// an owning [Vec<u8>].
///
/// Since this API remembers the underlying storage data in a view object, you don't have to pass it
/// in each time you're accessing a field. If you rather prefer an API that does not do that,
/// take a look at the [Field] API.
///
/// # Example:
/// ```
/// use binary_layout::prelude::*;
///
/// define_layout!(my_layout, LittleEndian, {
///   field_one: u16,
///   another_field: [u8; 16],
///   something_else: u32,
///   tail_data: [u8],
/// });
///
/// fn func(storage_data: &mut [u8]) {
///   let mut view = my_layout::View::new(storage_data);
///
///   // read some data
///   let format_version_header: u16 = view.field_one().read();
///   // equivalent: let format_version_header = u16::from_le_bytes((&storage_data[0..2]).try_into().unwrap());
///
///   // write some data
///   view.something_else_mut().write(10);
///   // equivalent: data_slice[18..22].copy_from_slice(&10u32.to_le_bytes());
///
///   // access a data region
///   let tail_data: &[u8] = view.tail_data().data();
///   // equivalent: let tail_data: &[u8] = &data_slice[22..];
///
///   // and modify it
///   view.tail_data_mut().data_mut()[..5].copy_from_slice(&[1, 2, 3, 4, 5]);
///   // equivalent: data_slice[18..22].copy_from_slice(&[1, 2, 3, 4, 5]);
/// }
/// ```
pub struct FieldView<S, F: IField> {
    storage: S,
    _p: PhantomData<F>,
}

// TODO We may be able to move FieldView into its own independent rust module now

impl<S, F: IField> FieldView<S, F> {
    /// Create a new view for a field over a given storage.
    /// You probably shouldn't call this directly but should instead call
    /// `your_layout::View::new()`, which is generated by the
    /// [define_layout!] macro for you.
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            _p: PhantomData,
        }
    }
}
impl<S: AsRef<[u8]>, F: IFieldCopyAccess> FieldView<S, F> {
    /// TODO Docs
    pub fn read(&self) -> F::HighLevelType {
        F::read(self.storage.as_ref())
    }
}
impl<S: AsMut<[u8]>, F: IFieldCopyAccess> FieldView<S, F> {
    /// TODO Docs
    pub fn write(&mut self, v: F::HighLevelType) {
        F::write(self.storage.as_mut(), v)
    }
}
impl<'a, S: 'a + AsRef<[u8]>, F: IFieldSliceAccess<'a>> FieldView<S, F> {
    /// TODO Docs
    pub fn data(&'a self) -> F::SliceType {
        F::data(self.storage.as_ref())
    }
}
impl<
        'a,
        S: ?Sized + AsRef<[u8]>,
        F: IFieldSliceAccess<'a, SliceType = &'a [u8], MutSliceType = &'a mut [u8]>,
    > FieldView<&'a S, F>
{
    /// Similar to [FieldView::data], but this also extracts the lifetime. The reference returned by [FieldView::data] can only life as long as the [FieldView] object lives.
    /// The reference returned by this function can live for as long as the original `packed_data` reference that as put into the [FieldView] lives.
    /// However, you can only call this if you let the [FieldView] die, it takes the `self` parameter by value.
    /// Also note that this function can only be called when the [FieldView] was constructed with either a `&[u8]` or a `&mut [u8]` as underlying storage for the `storage_data`.
    /// If the [FieldView] was constructed based on `Vec<u8>` storage, then this function semantically would have to return an owning subvector, but such a thing doesn't exist in Rust.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///     another_field: u64,
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &[u8]) -> &[u8] {
    ///     let view = my_layout::View::new(storage_data);
    ///     let tail_data: &[u8] = view.into_tail_data().extract();
    ///     // Now we return tail_data. Note that the view object doesn't survive
    ///     // this function but we can still return the `tail_data` reference.
    ///     // This wouldn't be possible with `FieldView::data`.
    ///     tail_data
    /// }
    /// ```
    pub fn extract(self) -> &'a [u8] {
        F::data(self.storage.as_ref())
    }
}
impl<
        'a,
        S: ?Sized + AsRef<[u8]>,
        F: IFieldSliceAccess<'a, SliceType = &'a [u8], MutSliceType = &'a mut [u8]>,
    > FieldView<&'a mut S, F>
{
    /// Similar to [FieldView::data], but this also extracts the lifetime. The reference returned by [FieldView::data] can only life as long as the [FieldView] object lives.
    /// The reference returned by this function can live for as long as the original `packed_data` reference that as put into the [FieldView] lives.
    /// However, you can only call this if you let the [FieldView] die, it takes the `self` parameter by value.
    /// Also note that this function can only be called when the [FieldView] was constructed with either a `&[u8]` or a `&mut [u8]` as underlying storage for the `storage_data`.
    /// If the [FieldView] was constructed based on `Vec<u8>` storage, then this function semantically would have to return an owning subvector, but such a thing doesn't exist in Rust.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///     another_field: u64,
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &[u8]) -> &[u8] {
    ///     let view = my_layout::View::new(storage_data);
    ///     let tail_data: &[u8] = view.into_tail_data().extract();
    ///     // Now we return tail_data. Note that the view object doesn't survive
    ///     // this function but we can still return the `tail_data` reference.
    ///     // This wouldn't be possible with `FieldView::data`.
    ///     tail_data
    /// }
    /// ```
    pub fn extract(self) -> &'a [u8] {
        let s: &'a S = self.storage;
        F::data(s.as_ref())
    }
}
impl<'a, S: 'a + AsMut<[u8]>, F: IFieldSliceAccess<'a>> FieldView<S, F> {
    /// TODO Docs
    pub fn data_mut(&'a mut self) -> F::MutSliceType {
        F::data_mut(self.storage.as_mut())
    }
}
impl<
        'a,
        S: ?Sized + AsMut<[u8]>,
        F: IFieldSliceAccess<'a, SliceType = &'a [u8], MutSliceType = &'a mut [u8]>,
    > FieldView<&'a mut S, F>
{
    /// Similar to [FieldView::data], but this also extracts the lifetime. The reference returned by [FieldView::data] can only life as long as the [FieldView] object lives.
    /// The reference returned by this function can live for as long as the original `packed_data` reference that as put into the [FieldView] lives.
    /// However, you can only call this if you let the [FieldView] die, it takes the `self` parameter by value.
    /// Also note that this function can only be called when the [FieldView] was constructed with either a `&[u8]` or a `&mut [u8]` as underlying storage for the `storage_data`.
    /// If the [FieldView] was constructed based on `Vec<u8>` storage, then this function semantically would have to return an owning subvector, but such a thing doesn't exist in Rust.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///     
    /// define_layout!(my_layout, LittleEndian, {
    ///     another_field: u64,
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &[u8]) -> &[u8] {
    ///     let view = my_layout::View::new(storage_data);
    ///     let tail_data: &[u8] = view.into_tail_data().extract();
    ///     // Now we return tail_data. Note that the view object doesn't survive
    ///     // this function but we can still return the `tail_data` reference.
    ///     // This wouldn't be possible with `FieldView::data`.
    ///     tail_data
    /// }
    /// ```
    pub fn extract_mut(self) -> &'a mut [u8] {
        F::data_mut(self.storage.as_mut())
    }
}

// trait FieldTypeAccessor {
//     type Field;
// }
// impl<S, F: FieldMetadata> FieldTypeAccessor for FieldView<S, F> {
//     type Field = F;
// }

// impl<T: ?Sized, E: Endianness, const OFFSET_: usize> FieldMetadata for Field<T, E, OFFSET_> {
//     type Type = T;
//     const OFFSET: usize = OFFSET_;
// }

// impl<T: FieldSize, E: Endianness, const OFFSET_: usize> SizedFieldMetadata
//     for Field<T, E, OFFSET_>
// {
//     const SIZE: usize = <T as FieldSize>::SIZE;
// }

macro_rules! int_field {
    ($type:ident) => {
        impl<E: Endianness, const OFFSET_: usize> IFieldCopyAccess for PrimitiveField<$type, E, OFFSET_> {
            // TODO Doc
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
                fn read(storage: &[u8]) -> $type {
                    let mut value = [0; std::mem::size_of::<$type>()];
                    value.copy_from_slice(
                        &storage.as_ref()[Self::OFFSET..(Self::OFFSET + std::mem::size_of::<$type>())],
                    );
                    match E::KIND {
                        EndianKind::Big => $type::from_be_bytes(value),
                        EndianKind::Little => $type::from_le_bytes(value),
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
                fn write(storage: &mut [u8], value: $type) {
                    let value_as_bytes = match E::KIND {
                        EndianKind::Big => value.to_be_bytes(),
                        EndianKind::Little => value.to_le_bytes(),
                    };
                    storage.as_mut()[Self::OFFSET..(Self::OFFSET + std::mem::size_of::<$type>())]
                        .copy_from_slice(&value_as_bytes);
                }
            }
        }

        impl<E: Endianness, const OFFSET_: usize> ISizedField for PrimitiveField<$type, E, OFFSET_> {
            // TODO Doc
            const SIZE: usize = std::mem::size_of::<$type>();
        }
        // impl FieldSize for $type {
        //     const SIZE: usize = std::mem::size_of::<$type>();
        // }
    };
}

int_field!(i8);
int_field!(i16);
int_field!(i32);
int_field!(i64);
int_field!(u8);
int_field!(u16);
int_field!(u32);
int_field!(u64);

/// Field type `[u8]`:
/// This field represents an [open ended byte array](crate#open-ended-byte-arrays-u8).
/// In this impl, we define accessors for such fields.
impl<'a, E: Endianness, const OFFSET_: usize> IFieldSliceAccess<'a>
    for PrimitiveField<[u8], E, OFFSET_>
{
    // TODO Docs
    type SliceType = &'a [u8];
    // TODO Docs
    type MutSliceType = &'a mut [u8];

    /// Borrow the data in the byte array with read access using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///
    /// define_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &[u8]) {
    ///     let tail_data: &[u8] = my_layout::tail_data::data(storage_data);
    /// }
    /// ```
    fn data(storage: &'a [u8]) -> &'a [u8] {
        &storage.as_ref()[Self::OFFSET..]
    }

    /// Borrow the data in the byte array with write access using the [Field] API.
    ///
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///     
    /// define_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     tail_data: [u8],
    /// });
    ///
    /// fn func(storage_data: &mut [u8]) {
    ///     let tail_data: &mut [u8] = my_layout::tail_data::data_mut(storage_data);
    /// }
    /// ```
    fn data_mut(storage: &'a mut [u8]) -> &'a mut [u8] {
        &mut storage.as_mut()[Self::OFFSET..]
    }
}

/// Field type `[u8; N]`:
/// This field represents a [fixed size byte array](crate#fixed-size-byte-arrays-u8-n).
/// In this impl, we define accessors for such fields.
impl<'a, E: Endianness, const N: usize, const OFFSET_: usize> IFieldSliceAccess<'a>
    for PrimitiveField<[u8; N], E, OFFSET_>
{
    // TODO Docs
    type SliceType = &'a [u8; N];
    // TODO Docs
    type MutSliceType = &'a mut [u8; N];

    /// Borrow the data in the byte array with read access using the [Field] API.
    ///  
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///     
    /// define_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     some_field: [u8; 5],
    ///     //... other fields
    /// });
    ///
    /// fn func(storage_data: &[u8]) {
    ///     let some_field: &[u8; 5] = my_layout::some_field::data(storage_data);
    /// }
    /// ```
    fn data(storage: &'a [u8]) -> &'a [u8; N] {
        <&[u8; N]>::try_from(&storage.as_ref()[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
    }

    /// Borrow the data in the byte array with write access using the [Field] API.
    ///  
    /// # Example:
    /// ```
    /// use binary_layout::prelude::*;
    ///     
    /// define_layout!(my_layout, LittleEndian, {
    ///     //... other fields ...
    ///     some_field: [u8; 5],
    ///     //... other fields
    /// });
    ///
    /// fn func(storage_data: &mut [u8]) {
    ///     let some_field: &mut [u8; 5] = my_layout::some_field::data_mut(storage_data);
    /// }
    /// ```
    fn data_mut(storage: &'a mut [u8]) -> &'a mut [u8; N] {
        <&mut [u8; N]>::try_from(&mut storage.as_mut()[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
    }
}
impl<E: Endianness, const N: usize, const OFFSET_: usize> ISizedField
    for PrimitiveField<[u8; N], E, OFFSET_>
{
    const SIZE: usize = N;
}
// impl<const N: usize> FieldSize for [u8; N] {
//     const SIZE: usize = N;
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_i8_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i8, LittleEndian, 5>;
        type Field2 = PrimitiveField<i8, LittleEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, -20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(-20, Field2::read(&storage));

        assert_eq!(50, i8::from_le_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            -20,
            i8::from_le_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, PrimitiveField::<i8, LittleEndian, 5>::SIZE);
        assert_eq!(1, PrimitiveField::<i8, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i8_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i8, BigEndian, 5>;
        type Field2 = PrimitiveField<i8, BigEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, -20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(-20, Field2::read(&storage));

        assert_eq!(50, i8::from_be_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            -20,
            i8::from_be_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, PrimitiveField::<i8, BigEndian, 5>::SIZE);
        assert_eq!(1, PrimitiveField::<i8, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_i16_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i16, LittleEndian, 5>;
        type Field2 = PrimitiveField<i16, LittleEndian, 20>;

        Field1::write(&mut storage, 500);
        Field2::write(&mut storage, -2000);

        assert_eq!(
            500,
            i16::from_le_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            -2000,
            i16::from_le_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(500, Field1::read(&storage));
        assert_eq!(-2000, Field2::read(&storage));

        assert_eq!(2, PrimitiveField::<i16, LittleEndian, 5>::SIZE);
        assert_eq!(2, PrimitiveField::<i16, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i16_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i16, BigEndian, 5>;
        type Field2 = PrimitiveField<i16, BigEndian, 20>;

        Field1::write(&mut storage, 500);
        Field2::write(&mut storage, -2000);

        assert_eq!(
            500,
            i16::from_be_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            -2000,
            i16::from_be_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(500, Field1::read(&storage));
        assert_eq!(-2000, Field2::read(&storage));

        assert_eq!(2, PrimitiveField::<i16, BigEndian, 5>::SIZE);
        assert_eq!(2, PrimitiveField::<i16, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_i32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i32, LittleEndian, 5>;
        type Field2 = PrimitiveField<i32, LittleEndian, 20>;

        Field1::write(&mut storage, 10i32.pow(8));
        Field2::write(&mut storage, -(10i32.pow(7)));

        assert_eq!(
            10i32.pow(8),
            i32::from_le_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            -(10i32.pow(7)),
            i32::from_le_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10i32.pow(8), Field1::read(&storage));
        assert_eq!(-(10i32.pow(7)), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<i32, LittleEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<i32, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i32, BigEndian, 5>;
        type Field2 = PrimitiveField<i32, BigEndian, 20>;

        Field1::write(&mut storage, 10i32.pow(8));
        Field2::write(&mut storage, -(10i32.pow(7)));

        assert_eq!(
            10i32.pow(8),
            i32::from_be_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            -(10i32.pow(7)),
            i32::from_be_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10i32.pow(8), Field1::read(&storage));
        assert_eq!(-(10i32.pow(7)), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<i32, BigEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<i32, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_i64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i64, LittleEndian, 5>;
        type Field2 = PrimitiveField<i64, LittleEndian, 20>;

        Field1::write(&mut storage, 10i64.pow(15));
        Field2::write(&mut storage, -(10i64.pow(14)));

        assert_eq!(
            10i64.pow(15),
            i64::from_le_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            -(10i64.pow(14)),
            i64::from_le_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10i64.pow(15), Field1::read(&storage));
        assert_eq!(-(10i64.pow(14)), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<i64, LittleEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<i64, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<i64, BigEndian, 5>;
        type Field2 = PrimitiveField<i64, BigEndian, 20>;

        Field1::write(&mut storage, 10i64.pow(15));
        Field2::write(&mut storage, -(10i64.pow(14)));

        assert_eq!(
            10i64.pow(15),
            i64::from_be_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            -(10i64.pow(14)),
            i64::from_be_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10i64.pow(15), Field1::read(&storage));
        assert_eq!(-(10i64.pow(14)), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<i64, BigEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<i64, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u8_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u8, LittleEndian, 5>;
        type Field2 = PrimitiveField<u8, LittleEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, 20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(20, Field2::read(&storage));

        assert_eq!(50, u8::from_le_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            20,
            u8::from_le_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, PrimitiveField::<u8, LittleEndian, 5>::SIZE);
        assert_eq!(1, PrimitiveField::<u8, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u8_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u8, BigEndian, 5>;
        type Field2 = PrimitiveField<u8, BigEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, 20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(20, Field2::read(&storage));

        assert_eq!(50, u8::from_be_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            20,
            u8::from_be_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, PrimitiveField::<u8, BigEndian, 5>::SIZE);
        assert_eq!(1, PrimitiveField::<u8, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u16_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u16, LittleEndian, 5>;
        type Field2 = PrimitiveField<u16, LittleEndian, 20>;

        Field1::write(&mut storage, 500);
        Field2::write(&mut storage, 2000);

        assert_eq!(
            500,
            u16::from_le_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            2000,
            u16::from_le_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(500, Field1::read(&storage));
        assert_eq!(2000, Field2::read(&storage));

        assert_eq!(2, PrimitiveField::<u16, LittleEndian, 5>::SIZE);
        assert_eq!(2, PrimitiveField::<u16, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u16_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u16, BigEndian, 5>;
        type Field2 = PrimitiveField<u16, BigEndian, 20>;

        Field1::write(&mut storage, 500);
        Field2::write(&mut storage, 2000);

        assert_eq!(
            500,
            u16::from_be_bytes((&storage[5..7]).try_into().unwrap())
        );
        assert_eq!(
            2000,
            u16::from_be_bytes((&storage[20..22]).try_into().unwrap())
        );

        assert_eq!(500, Field1::read(&storage));
        assert_eq!(2000, Field2::read(&storage));

        assert_eq!(2, PrimitiveField::<u16, BigEndian, 5>::SIZE);
        assert_eq!(2, PrimitiveField::<u16, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u32, LittleEndian, 5>;
        type Field2 = PrimitiveField<u32, LittleEndian, 20>;

        Field1::write(&mut storage, 10u32.pow(8));
        Field2::write(&mut storage, 10u32.pow(7));

        assert_eq!(
            10u32.pow(8),
            u32::from_le_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10u32.pow(7),
            u32::from_le_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10u32.pow(8), Field1::read(&storage));
        assert_eq!(10u32.pow(7), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<u32, LittleEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<u32, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u32, BigEndian, 5>;
        type Field2 = PrimitiveField<u32, BigEndian, 20>;

        Field1::write(&mut storage, 10u32.pow(8));
        Field2::write(&mut storage, 10u32.pow(7));

        assert_eq!(
            10u32.pow(8),
            u32::from_be_bytes((&storage[5..9]).try_into().unwrap())
        );
        assert_eq!(
            10u32.pow(7),
            u32::from_be_bytes((&storage[20..24]).try_into().unwrap())
        );

        assert_eq!(10u32.pow(8), Field1::read(&storage));
        assert_eq!(10u32.pow(7), Field2::read(&storage));

        assert_eq!(4, PrimitiveField::<u32, BigEndian, 5>::SIZE);
        assert_eq!(4, PrimitiveField::<u32, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u64, LittleEndian, 5>;
        type Field2 = PrimitiveField<u64, LittleEndian, 20>;

        Field1::write(&mut storage, 10u64.pow(15));
        Field2::write(&mut storage, 10u64.pow(14));

        assert_eq!(
            10u64.pow(15),
            u64::from_le_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10u64.pow(14),
            u64::from_le_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10u64.pow(15), Field1::read(&storage));
        assert_eq!(10u64.pow(14), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<u64, LittleEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<u64, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<u64, BigEndian, 5>;
        type Field2 = PrimitiveField<u64, BigEndian, 20>;

        Field1::write(&mut storage, 10u64.pow(15));
        Field2::write(&mut storage, 10u64.pow(14));

        assert_eq!(
            10u64.pow(15),
            u64::from_be_bytes((&storage[5..13]).try_into().unwrap())
        );
        assert_eq!(
            10u64.pow(14),
            u64::from_be_bytes((&storage[20..28]).try_into().unwrap())
        );

        assert_eq!(10u64.pow(15), Field1::read(&storage));
        assert_eq!(10u64.pow(14), Field2::read(&storage));

        assert_eq!(8, PrimitiveField::<u64, BigEndian, 5>::SIZE);
        assert_eq!(8, PrimitiveField::<u64, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_slice() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<[u8], LittleEndian, 5>;
        type Field2 = PrimitiveField<[u8], BigEndian, 7>;

        Field1::data_mut(&mut storage)[..5].copy_from_slice(&[10, 20, 30, 40, 50]);
        Field2::data_mut(&mut storage)[..5].copy_from_slice(&[60, 70, 80, 90, 100]);

        assert_eq!(&[10, 20, 60, 70, 80], &Field1::data(&storage)[..5]);
        assert_eq!(&[60, 70, 80, 90, 100], &Field2::data(&storage)[..5]);
    }

    #[test]
    fn test_array() {
        let mut storage = vec![0; 1024];

        type Field1 = PrimitiveField<[u8; 2], LittleEndian, 5>;
        type Field2 = PrimitiveField<[u8; 5], BigEndian, 6>;

        Field1::data_mut(&mut storage).copy_from_slice(&[10, 20]);
        Field2::data_mut(&mut storage).copy_from_slice(&[60, 70, 80, 90, 100]);

        assert_eq!(&[10, 60], Field1::data(&storage));
        assert_eq!(&[60, 70, 80, 90, 100], Field2::data(&storage));

        assert_eq!(2, PrimitiveField::<[u8; 2], LittleEndian, 5>::SIZE);
        assert_eq!(5, PrimitiveField::<[u8; 5], BigEndian, 5>::SIZE);
    }
}
