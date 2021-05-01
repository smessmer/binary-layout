use std::convert::TryFrom;
use std::marker::PhantomData;

// TODO With const_evaluatable_checked, FieldSize and Field could be merged by adding a size() function to Field,
// but we'd need https://github.com/rust-lang/rust/issues/76560 first.
pub trait FieldSize {
    const SIZE: usize;
}

/// An enum representing the endianness used in a layout for accessing primitive integer fields.
pub enum EndianKind {
    Big,
    Little,
}

/// This marker trait represents the endianness used in a layout for accessing primitive integer fields.
pub trait Endianness {
    const KIND: EndianKind;
}

/// This is a marker type to mark layouts using big endian encoding
pub struct BigEndian {}
impl Endianness for BigEndian {
    const KIND: EndianKind = EndianKind::Big;
}

/// This is a marker type to mark layouts using little endian encoding
pub struct LittleEndian {}
impl Endianness for LittleEndian {
    const KIND: EndianKind = EndianKind::Little;
}

/// This trait offers access to the metadata of a field in a layout
pub trait FieldMetadata {
    /// The data type of the field, e.g. [u8], [i32], ...
    type Type: ?Sized;

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
    /// fn main() {
    ///     assert_eq!(0, my_layout::field1::OFFSET);
    ///     assert_eq!(2, my_layout::field2::OFFSET);
    ///     assert_eq!(6, my_layout::field3::OFFSET);
    /// }
    /// ```
    const OFFSET: usize;
}

/// This trait offers access to the metadata of a sized field in a layout.
/// Sized fields are all fields with a defined size. This is almost all fields.
/// The only exception is an unsized array field that can be used to match
/// tail data, i.e. any data at the end of the storage after all other fields
/// were defined and until the storage ends.
pub trait SizedFieldMetadata {
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
    /// fn main() {
    ///     assert_eq!(2, my_layout::field1::SIZE);
    ///     assert_eq!(4, my_layout::field2::SIZE);
    ///     assert_eq!(1, my_layout::field3::SIZE);
    /// }
    /// ```
    const SIZE: usize;
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
pub struct Field<T: ?Sized, E: Endianness, const OFFSET_: usize> {
    _p1: PhantomData<T>,
    _p2: PhantomData<E>,
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
pub struct FieldView<S, F: FieldMetadata> {
    storage: S,
    _p: PhantomData<F>,
}

impl<S, F: FieldMetadata> FieldView<S, F> {
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

trait FieldTypeAccessor {
    type Field;
}
impl<S, F: FieldMetadata> FieldTypeAccessor for FieldView<S, F> {
    type Field = F;
}

impl<T: ?Sized, E: Endianness, const OFFSET_: usize> FieldMetadata for Field<T, E, OFFSET_> {
    type Type = T;
    const OFFSET: usize = OFFSET_;
}

impl<T: FieldSize, E: Endianness, const OFFSET_: usize> SizedFieldMetadata
    for Field<T, E, OFFSET_>
{
    const SIZE: usize = <T as FieldSize>::SIZE;
}

macro_rules! int_field {
    ($type:ident) => {
        doc_comment::doc_comment! {
            concat! {
                "Field type [", stringify!($type), "]: ",
                "This field represents a primitive integer. In this impl, we define read accessors for such integer fields. See [supported primitive integer types](crate#primitive-integer-types)."
            },
            impl<E: Endianness, const OFFSET_: usize> Field<$type, E, OFFSET_> {
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
                    #[allow(dead_code)]
                    pub fn read(storage: &[u8]) -> $type {
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
            }
        }
        impl FieldSize for $type {
            const SIZE: usize = std::mem::size_of::<$type>();
        }
        doc_comment::doc_comment! {
            concat! {
                "Field type [", stringify!($type), "]: ",
                "This field represents a little endian integer. In this impl, we define write accessors for such integer fields. See [supported primitive integer types](crate#primitive-integer-types).",
            },
            impl<E: Endianness, const OFFSET_: usize> Field<$type, E, OFFSET_> {
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
                    #[allow(dead_code)]
                    pub fn write(storage: &mut [u8], value: $type) {
                        let value_as_bytes = match E::KIND {
                            EndianKind::Big => value.to_be_bytes(),
                            EndianKind::Little => value.to_le_bytes(),
                        };
                        storage.as_mut()[Self::OFFSET..(Self::OFFSET + std::mem::size_of::<$type>())]
                            .copy_from_slice(&value_as_bytes);
                    }
                }
            }
        }
        doc_comment::doc_comment! {
            concat! {
                "Field type [", stringify!($type), "]: ",
                "This field represents a little endian integer. In this impl, we define read accessors for such integer fields. See [supported primitive integer types](crate#primitive-integer-types).",
            },
            impl<S: AsRef<[u8]>, E: Endianness, const OFFSET_: usize> FieldView<S, Field<$type, E, OFFSET_>> {
                doc_comment::doc_comment! {
                    concat! {"
                    Read the integer field from a given data region, assuming the defined layout, using the [FieldView] API.
                    
                    # Example:
                    
                    ```
                    use binary_layout::prelude::*;
                        
                    define_layout!(my_layout, LittleEndian, {
                       //... other fields ...
                       some_integer_field: ", stringify!($type), "
                       //... other fields ...
                    });

                    fn func(storage_data: &[u8]) {
                        let view = my_layout::View::new(storage_data);
                        let read: ", stringify!($type), " = view.some_integer_field().read();
                    }
                    ```
                    "},
                    #[allow(dead_code)]
                    pub fn read(&self) -> $type {
                        <Self as FieldTypeAccessor>::Field::read(self.storage.as_ref())
                    }
                }
            }
        }
        doc_comment::doc_comment! {
            concat! {
                "Field type [", stringify!($type), "]: ",
                "This field represents a little endian integer. In this impl, we define write accessors for such integer fields. See [supported primitive integer types](crate#primitive-integer-types).",
            },
            impl<S: AsMut<[u8]>, E: Endianness, const OFFSET_: usize> FieldView<S, Field<$type, E, OFFSET_>> {
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
                        let mut view = my_layout::View::new(storage_data);
                        view.some_integer_field_mut().write(10);
                    }
                    ```
                    "},
                    #[allow(dead_code)]
                    pub fn write(&mut self, value: $type) {
                        <Self as FieldTypeAccessor>::Field::write(self.storage.as_mut(), value);
                    }
                }
            }
        }
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
/// In this impl, we define read accessors for such fields.
impl<E: Endianness, const OFFSET_: usize> Field<[u8], E, OFFSET_> {
    doc_comment::doc_comment! {
        concat! {"
        Borrow the data in the byte array with read access using the [Field] API.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           //... other fields ...
           tail_data: [u8],
        });

        fn func(storage_data: &[u8]) {
            let tail_data: &[u8] = my_layout::tail_data::data(storage_data);
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn data(storage: &[u8]) -> &[u8] {
            &storage.as_ref()[Self::OFFSET..]
        }
    }
}
/// Field type `[u8]`:
/// This field represents an [open ended byte array](crate#open-ended-byte-arrays-u8).
/// In this impl, we define read accessors for such fields.
impl<E: Endianness, const OFFSET_: usize> Field<[u8], E, OFFSET_> {
    doc_comment::doc_comment! {
        concat! {"
        Borrow the data in the byte array with write access using the [Field] API.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           //... other fields ...
           tail_data: [u8],
        });

        fn func(storage_data: &mut [u8]) {
            let tail_data: &mut [u8] = my_layout::tail_data::data_mut(storage_data);
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn data_mut(storage: &mut [u8]) -> &mut [u8] {
            &mut storage.as_mut()[Self::OFFSET..]
        }
    }
}
/// Field type `[u8]`:
/// This field represents an [open ended byte array](crate#open-ended-byte-arrays-u8).
/// In this impl, we define accessors that transfer ownership of the underlying immutable package data for such fields.
impl<S: AsRef<[u8]>, E: Endianness, const OFFSET_: usize> FieldView<S, Field<[u8], E, OFFSET_>> {
    doc_comment::doc_comment! {
        concat! {"
        Borrow the data in the byte array with read access using the [FieldView] API.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           //... other fields ...
           tail_data: [u8],
        });

        fn func(storage_data: &[u8]) {
            let view = my_layout::View::new(storage_data);
            let tail_data: &[u8] = view.tail_data().data();
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn data(&self) -> &[u8] {
            <Self as FieldTypeAccessor>::Field::data(self.storage.as_ref())
        }
    }
}
/// Field type `[u8]`:
/// This field represents an [open ended byte array](crate#open-ended-byte-arrays-u8).
/// In this impl, we define write accessors for such fields.
impl<S: AsMut<[u8]>, E: Endianness, const OFFSET_: usize> FieldView<S, Field<[u8], E, OFFSET_>> {
    doc_comment::doc_comment! {
        concat! {"
        Borrow the data in the byte array with write access using the [FieldView] API.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           //... other fields ...
           tail_data: [u8],
        });

        fn func(storage_data: &mut [u8]) {
            let mut view = my_layout::View::new(storage_data);
            let tail_data: &mut [u8] = view.tail_data_mut().data_mut();
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn data_mut(&mut self) -> &mut [u8] {
            <Self as FieldTypeAccessor>::Field::data_mut(self.storage.as_mut())
        }
    }
}
/// Field type `[u8]`:
/// This field represents an [open ended byte array](crate#open-ended-byte-arrays-u8).
/// In this impl, we define read accessors for such fields.
impl<'a, S: AsRef<[u8]> + ?Sized, E: Endianness, const OFFSET_: usize>
    FieldView<&'a S, Field<[u8], E, OFFSET_>>
{
    doc_comment::doc_comment! {
        concat! {"
        Similar to [FieldView::data], but this also extracts the lifetime. The reference returned by [FieldView::data] can only life as long as the [FieldView] object lives.
        The reference returned by this function can live for as long as the original `packed_data` reference that as put into the [FieldView] lives.
        However, you can only call this if you let the [FieldView] die, it takes the `self` parameter by value.
        Also note that this function can only be called when the [FieldView] was constructed with either a `&[u8]` or a `&mut [u8]` as underlying storage for the `storage_data`.
        If the [FieldView] was constructed based on `Vec<u8>` storage, then this function semantically would have to return an owning subvector, but such a thing doesn't exist in Rust.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           another_field: u64,
           tail_data: [u8],
        });

        fn func(storage_data: &[u8]) -> &[u8] {
            let view = my_layout::View::new(storage_data);
            let tail_data: &[u8] = view.into_tail_data().extract();
            // Now we return tail_data. Note that the view object doesn't survive
            // this function but we can still return the `tail_data` reference.
            // This wouldn't be possible with `FieldView::data`.
            tail_data
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn extract(self) -> &'a [u8] {
            <Self as FieldTypeAccessor>::Field::data(self.storage.as_ref())
        }
    }
}
/// Field type `[u8]`:
/// This field represents an [open ended byte array](crate#open-ended-byte-arrays-u8).
/// In this impl, we define accessors that transfer ownership of the underlying mutable package data for such fields.
impl<'a, S: AsMut<[u8]> + ?Sized, E: Endianness, const OFFSET_: usize>
    FieldView<&'a mut S, Field<[u8], E, OFFSET_>>
{
    doc_comment::doc_comment! {
        concat! {"
        Similar to [FieldView::data], but this also extracts the lifetime. The reference returned by [FieldView::data] can only life as long as the [FieldView] object lives.
        The reference returned by this function can live for as long as the original `packed_data` reference that as put into the [FieldView] lives.
        However, you can only call this if you let the [FieldView] die, it takes the `self` parameter by value.
        Also note that this function can only be called when the [FieldView] was constructed with either a `&[u8]` or a `&mut [u8]` as underlying storage for the `storage_data`.
        If the [FieldView] was constructed based on `Vec<u8>` storage, then this function semantically would have to return an owning subvector, but such a thing doesn't exist in Rust.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           another_field: u64,
           tail_data: [u8],
        });

        fn func(storage_data: &[u8]) -> &[u8] {
            let view = my_layout::View::new(storage_data);
            let tail_data: &[u8] = view.into_tail_data().extract();
            // Now we return tail_data. Note that the view object doesn't survive
            // this function but we can still return the `tail_data` reference.
            // This wouldn't be possible with `FieldView::data`.
            tail_data
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn extract(self) -> &'a mut [u8] {
            <Self as FieldTypeAccessor>::Field::data_mut(self.storage.as_mut())
        }
    }
}

/// Field type `[u8; N]`:
/// This field represents a [fixed size byte array](crate#fixed-size-byte-arrays-u8-n).
/// In this impl, we define read accessors for such fields.
impl<E: Endianness, const N: usize, const OFFSET_: usize> Field<[u8; N], E, OFFSET_> {
    doc_comment::doc_comment! {
        concat! {"
        Borrow the data in the byte array with read access using the [Field] API.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           //... other fields ...
           some_field: [u8; 5],
           //... other fields
        });

        fn func(storage_data: &[u8]) {
            let some_field: &[u8; 5] = my_layout::some_field::data(storage_data);
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn data(storage: &[u8]) -> &[u8; N] {
            <&[u8; N]>::try_from(&storage.as_ref()[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
        }
    }
}
/// Field type `[u8; N]`:
/// This field represents a [fixed size byte array](crate#fixed-size-byte-arrays-u8-n).
/// In this impl, we define write accessors for such fields.
impl<E: Endianness, const N: usize, const OFFSET_: usize> Field<[u8; N], E, OFFSET_> {
    doc_comment::doc_comment! {
        concat! {"
        Borrow the data in the byte array with write access using the [Field] API.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           //... other fields ...
           some_field: [u8; 5],
           //... other fields
        });

        fn func(storage_data: &mut [u8]) {
            let some_field: &mut [u8; 5] = my_layout::some_field::data_mut(storage_data);
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn data_mut(storage: &mut [u8]) -> &mut [u8; N] {
            <&mut [u8; N]>::try_from(&mut storage.as_mut()[Self::OFFSET..(Self::OFFSET + N)]).unwrap()
        }
    }
}
impl<const N: usize> FieldSize for [u8; N] {
    const SIZE: usize = N;
}
/// Field type `[u8; N]`:
/// This field represents a [fixed size byte array](crate#fixed-size-byte-arrays-u8-n).
/// In this impl, we define read accessors for such fields.
impl<S: AsRef<[u8]>, E: Endianness, const N: usize, const OFFSET_: usize>
    FieldView<S, Field<[u8; N], E, OFFSET_>>
{
    doc_comment::doc_comment! {
        concat! {"
        Borrow the data in the byte array with read access using the [FieldView] API.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           //... other fields ...
           some_field: [u8; 5],
           //... other fields
        });

        fn func(storage_data: &[u8]) {
            let view = my_layout::View::new(storage_data);
            let some_field: &[u8; 5] = view.some_field().data();
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn data(&self) -> &[u8; N] {
            <Self as FieldTypeAccessor>::Field::data(self.storage.as_ref())
        }
    }
}
/// Field type `[u8; N]`:
/// This field represents a [fixed size byte array](crate#fixed-size-byte-arrays-u8-n).
/// In this impl, we define write accessors for such fields.
impl<S: AsMut<[u8]>, E: Endianness, const N: usize, const OFFSET_: usize>
    FieldView<S, Field<[u8; N], E, OFFSET_>>
{
    doc_comment::doc_comment! {
        concat! {"
        Borrow the data in the byte array with write access using the [FieldView] API.
        
        # Example:
        
        ```
        use binary_layout::prelude::*;
            
        define_layout!(my_layout, LittleEndian, {
           //... other fields ...
           some_field: [u8; 5],
           //... other fields
        });

        fn func(storage_data: &mut [u8]) {
            let mut view = my_layout::View::new(storage_data);
            let some_field: &mut [u8; 5] = view.some_field_mut().data_mut();
        }
        ```
        "},
        #[allow(dead_code)]
        pub fn data_mut(&mut self) -> &mut [u8; N] {
            <Self as FieldTypeAccessor>::Field::data_mut(self.storage.as_mut())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_i8_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<i8, LittleEndian, 5>;
        type Field2 = Field<i8, LittleEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, -20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(-20, Field2::read(&storage));

        assert_eq!(50, i8::from_le_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            -20,
            i8::from_le_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, Field::<i8, LittleEndian, 5>::SIZE);
        assert_eq!(1, Field::<i8, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i8_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<i8, BigEndian, 5>;
        type Field2 = Field<i8, BigEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, -20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(-20, Field2::read(&storage));

        assert_eq!(50, i8::from_be_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            -20,
            i8::from_be_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, Field::<i8, BigEndian, 5>::SIZE);
        assert_eq!(1, Field::<i8, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_i16_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<i16, LittleEndian, 5>;
        type Field2 = Field<i16, LittleEndian, 20>;

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

        assert_eq!(2, Field::<i16, LittleEndian, 5>::SIZE);
        assert_eq!(2, Field::<i16, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i16_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<i16, BigEndian, 5>;
        type Field2 = Field<i16, BigEndian, 20>;

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

        assert_eq!(2, Field::<i16, BigEndian, 5>::SIZE);
        assert_eq!(2, Field::<i16, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_i32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<i32, LittleEndian, 5>;
        type Field2 = Field<i32, LittleEndian, 20>;

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
        assert_eq!(-10i32.pow(7), Field2::read(&storage));

        assert_eq!(4, Field::<i32, LittleEndian, 5>::SIZE);
        assert_eq!(4, Field::<i32, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<i32, BigEndian, 5>;
        type Field2 = Field<i32, BigEndian, 20>;

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
        assert_eq!(-10i32.pow(7), Field2::read(&storage));

        assert_eq!(4, Field::<i32, BigEndian, 5>::SIZE);
        assert_eq!(4, Field::<i32, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_i64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<i64, LittleEndian, 5>;
        type Field2 = Field<i64, LittleEndian, 20>;

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
        assert_eq!(-10i64.pow(14), Field2::read(&storage));

        assert_eq!(8, Field::<i64, LittleEndian, 5>::SIZE);
        assert_eq!(8, Field::<i64, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_i64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<i64, BigEndian, 5>;
        type Field2 = Field<i64, BigEndian, 20>;

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
        assert_eq!(-10i64.pow(14), Field2::read(&storage));

        assert_eq!(8, Field::<i64, BigEndian, 5>::SIZE);
        assert_eq!(8, Field::<i64, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u8_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<u8, LittleEndian, 5>;
        type Field2 = Field<u8, LittleEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, 20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(20, Field2::read(&storage));

        assert_eq!(50, u8::from_le_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            20,
            u8::from_le_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, Field::<u8, LittleEndian, 5>::SIZE);
        assert_eq!(1, Field::<u8, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u8_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<u8, BigEndian, 5>;
        type Field2 = Field<u8, BigEndian, 20>;

        Field1::write(&mut storage, 50);
        Field2::write(&mut storage, 20);

        assert_eq!(50, Field1::read(&storage));
        assert_eq!(20, Field2::read(&storage));

        assert_eq!(50, u8::from_be_bytes((&storage[5..6]).try_into().unwrap()));
        assert_eq!(
            20,
            u8::from_be_bytes((&storage[20..21]).try_into().unwrap())
        );

        assert_eq!(1, Field::<u8, BigEndian, 5>::SIZE);
        assert_eq!(1, Field::<u8, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u16_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<u16, LittleEndian, 5>;
        type Field2 = Field<u16, LittleEndian, 20>;

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

        assert_eq!(2, Field::<u16, LittleEndian, 5>::SIZE);
        assert_eq!(2, Field::<u16, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u16_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<u16, BigEndian, 5>;
        type Field2 = Field<u16, BigEndian, 20>;

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

        assert_eq!(2, Field::<u16, BigEndian, 5>::SIZE);
        assert_eq!(2, Field::<u16, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u32_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<u32, LittleEndian, 5>;
        type Field2 = Field<u32, LittleEndian, 20>;

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

        assert_eq!(4, Field::<u32, LittleEndian, 5>::SIZE);
        assert_eq!(4, Field::<u32, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u32_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<u32, BigEndian, 5>;
        type Field2 = Field<u32, BigEndian, 20>;

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

        assert_eq!(4, Field::<u32, BigEndian, 5>::SIZE);
        assert_eq!(4, Field::<u32, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_u64_littleendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<u64, LittleEndian, 5>;
        type Field2 = Field<u64, LittleEndian, 20>;

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

        assert_eq!(8, Field::<u64, LittleEndian, 5>::SIZE);
        assert_eq!(8, Field::<u64, LittleEndian, 5>::SIZE);
    }

    #[test]
    fn test_u64_bigendian() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<u64, BigEndian, 5>;
        type Field2 = Field<u64, BigEndian, 20>;

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

        assert_eq!(8, Field::<u64, BigEndian, 5>::SIZE);
        assert_eq!(8, Field::<u64, BigEndian, 5>::SIZE);
    }

    #[test]
    fn test_slice() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<[u8], LittleEndian, 5>;
        type Field2 = Field<[u8], BigEndian, 7>;

        Field1::data_mut(&mut storage)[..5].copy_from_slice(&[10, 20, 30, 40, 50]);
        Field2::data_mut(&mut storage)[..5].copy_from_slice(&[60, 70, 80, 90, 100]);

        assert_eq!(&[10, 20, 60, 70, 80], &Field1::data(&storage)[..5]);
        assert_eq!(&[60, 70, 80, 90, 100], &Field2::data(&storage)[..5]);
    }

    #[test]
    fn test_array() {
        let mut storage = vec![0; 1024];

        type Field1 = Field<[u8; 2], LittleEndian, 5>;
        type Field2 = Field<[u8; 5], BigEndian, 6>;

        Field1::data_mut(&mut storage).copy_from_slice(&[10, 20]);
        Field2::data_mut(&mut storage).copy_from_slice(&[60, 70, 80, 90, 100]);

        assert_eq!(&[10, 60], Field1::data(&storage));
        assert_eq!(&[60, 70, 80, 90, 100], Field2::data(&storage));

        assert_eq!(2, Field::<[u8; 2], LittleEndian, 5>::SIZE);
        assert_eq!(5, Field::<[u8; 5], BigEndian, 5>::SIZE);
    }
}
