// TODO Document.
// - Also document the View class that gets generated. Since they probably don't see the rustdoc for it in the package_view crate, let's also add its documentation to the define_layout! documentation
// - Document different styles of storage: &[u8], &mut [u8], Vec<u8>
#[macro_export]
macro_rules! define_layout {
    ($name: ident, $endianness: ident, {$($field_name: ident : $field_type: ty),* $(,)?}) => {
        #[allow(dead_code)]
        mod $name {
            #[allow(unused_imports)]
            use super::*;

            $crate::define_layout!(_impl_fields $crate::$endianness, 0, {$($field_name : $field_type),*});

            pub struct View<S> {
                storage: S,
            }
            impl <S> View<S> {
                pub fn new(storage: S) -> Self {
                    Self {storage}
                }

                $crate::define_layout!(_impl_view_into {$($field_name),*});
            }
            impl <S: AsRef<[u8]>> View<S> {
                $crate::define_layout!(_impl_view_asref {$($field_name),*});
            }
            impl <S: AsMut<[u8]>> View<S> {
                $crate::define_layout!(_impl_view_asmut {$($field_name),*});
            }
        }
    };

    (_impl_fields $endianness: ty, $offset_accumulator: expr, {}) => {};
    (_impl_fields $endianness: ty, $offset_accumulator: expr, {$name: ident : $type: ty $(, $name_tail: ident : $type_tail: ty)*}) => {
        #[allow(non_camel_case_types)]
        pub type $name = $crate::Field::<$type, $endianness, $offset_accumulator>;
        $crate::define_layout!(_impl_fields $endianness, {($offset_accumulator + <$type as $crate::FieldSize>::SIZE)}, {$($name_tail : $type_tail),*});
    };

    (_impl_view_asref {}) => {};
    (_impl_view_asref {$name: ident $(, $name_tail: ident)*}) => {
        pub fn $name(&self) -> $crate::FieldView::<&[u8], $name> {
            $crate::FieldView::new(self.storage.as_ref())
        }
        $crate::define_layout!(_impl_view_asref {$($name_tail),*});
    };

    (_impl_view_asmut {}) => {};
    (_impl_view_asmut {$name: ident $(, $name_tail: ident)*}) => {
        paste::paste!{
            pub fn [<$name _mut>](&mut self) -> $crate::FieldView::<&mut [u8], $name> {
                $crate::FieldView::new(self.storage.as_mut())
            }
        }
        $crate::define_layout!(_impl_view_asmut {$($name_tail),*});
    };

    (_impl_view_into {}) => {};
    (_impl_view_into {$name: ident $(, $name_tail: ident)*}) => {
        paste::paste!{
            pub fn [<into_ $name>](self) -> $crate::FieldView::<S, $name> {
                $crate::FieldView::new(self.storage)
            }
        }
        $crate::define_layout!(_impl_view_into {$($name_tail),*});
    };
}

#[cfg(test)]
mod tests {
    use crate::{FieldMetadata, SizedFieldMetadata};

    use rand::{rngs::StdRng, RngCore, SeedableRng};
    use std::convert::TryInto;

    fn data_region(size: usize, seed: u64) -> Vec<u8> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut res = vec![0; size];
        rng.fill_bytes(&mut res);
        res
    }

    #[test]
    fn test_layout_empty() {
        define_layout!(empty, LittleEndian, {});
    }

    mod sliceonly {
        use super::*;
        define_layout!(sliceonly, LittleEndian, { field: [u8] });

        #[test]
        fn fields() {
            assert_eq!(0, sliceonly::field::OFFSET);
            let mut storage = data_region(1024, 5);
            assert_eq!(&data_region(1024, 5), sliceonly::field::data(&storage));

            sliceonly::field::data_mut(&mut storage).copy_from_slice(&data_region(1024, 6));
            assert_eq!(&data_region(1024, 6), sliceonly::field::data(&storage));
        }

        #[test]
        fn view_readonly() {
            let storage = data_region(1024, 5);
            let view = sliceonly::View::new(&storage);
            assert_eq!(&data_region(1024, 5), view.field().data());
        }

        #[test]
        fn view_readwrite() {
            let mut storage = data_region(1024, 5);
            let mut view = sliceonly::View::new(&mut storage);
            assert_eq!(&data_region(1024, 5), view.field().data());

            assert_eq!(&data_region(1024, 5), view.field().data());
            view.field_mut()
                .data_mut()
                .copy_from_slice(&data_region(1024, 6));
            assert_eq!(&data_region(1024, 6), view.field().data());
        }

        #[test]
        fn view_vec() {
            let mut view = sliceonly::View::new(data_region(1024, 5));

            assert_eq!(&data_region(1024, 5), view.field().data());
            view.field_mut()
                .data_mut()
                .copy_from_slice(&data_region(1024, 6));
            assert_eq!(&data_region(1024, 6), view.field().data());
        }
    }

    mod noslice {
        use super::*;

        define_layout!(noslice, LittleEndian, {
            first: i8,
            second: i64,
            third: u16,
        });

        #[test]
        fn fields() {
            let mut storage = data_region(1024, 5);

            assert_eq!(0, noslice::first::OFFSET);
            assert_eq!(1, noslice::first::SIZE);
            assert_eq!(1, noslice::second::OFFSET);
            assert_eq!(8, noslice::second::SIZE);
            assert_eq!(9, noslice::third::OFFSET);
            assert_eq!(2, noslice::third::SIZE);

            noslice::first::write(&mut storage, 60);
            noslice::second::write(&mut storage, -100_000_000_000);
            noslice::third::write(&mut storage, 1_000);

            assert_eq!(60, noslice::first::read(&storage));
            assert_eq!(-100_000_000_000, noslice::second::read(&storage));
            assert_eq!(1_000, noslice::third::read(&storage));
        }

        // TODO view_readonly
        // TODO view_readwrite
        // TODO view_vec
    }

    mod withslice {
        use super::*;
        define_layout!(withslice, LittleEndian, {
            first: i8,
            second: i64,
            third: [u8; 5],
            fourth: u16,
            fifth: [u8],
        });

        #[test]
        fn fields() {
            let mut storage = data_region(1024, 5);

            assert_eq!(0, withslice::first::OFFSET);
            assert_eq!(1, withslice::first::SIZE);
            assert_eq!(1, withslice::second::OFFSET);
            assert_eq!(8, withslice::second::SIZE);
            assert_eq!(9, withslice::third::OFFSET);
            assert_eq!(5, withslice::third::SIZE);
            assert_eq!(14, withslice::fourth::OFFSET);
            assert_eq!(2, withslice::fourth::SIZE);
            assert_eq!(16, withslice::fifth::OFFSET);
            assert_eq!(5, withslice::third::data(&storage).len());
            assert_eq!(5, withslice::third::data_mut(&mut storage).len());
            assert_eq!(1024 - 16, withslice::fifth::data(&storage).len());
            assert_eq!(1024 - 16, withslice::fifth::data_mut(&mut storage).len());

            withslice::first::write(&mut storage, 60);
            withslice::second::write(&mut storage, -100_000_000_000);
            withslice::third::data_mut(&mut storage).copy_from_slice(&[10, 20, 30, 40, 50]);
            withslice::fourth::write(&mut storage, 1_000);
            withslice::fifth::data_mut(&mut storage).copy_from_slice(&data_region(1024 - 16, 6));

            assert_eq!(60, withslice::first::read(&storage));
            assert_eq!(-100_000_000_000, withslice::second::read(&storage));
            assert_eq!(&[10, 20, 30, 40, 50], withslice::third::data(&storage));
            assert_eq!(1_000, withslice::fourth::read(&storage));
            assert_eq!(&data_region(1024 - 16, 6), withslice::fifth::data(&storage));
        }

        // TODO view_readonly
        // TODO view_readwrite
        // TODO view_vec
    }

    #[test]
    fn can_be_created_with_and_without_trailing_comma() {
        define_layout!(first, LittleEndian, { field: u8 });
        define_layout!(second, LittleEndian, {
            field: u8,
            second: u16
        });
        define_layout!(third, LittleEndian, {
            field: u8,
        });
        define_layout!(fourth, LittleEndian, {
            field: u8,
            second: u16,
        });
    }

    #[test]
    fn given_immutableview_when_extractingimmutableref() {
        define_layout!(layout, LittleEndian, {
            field: u8,
            tail: [u8],
        });

        let storage = data_region(1024, 0);
        let extracted: &[u8] = {
            let view = layout::View::new(&storage);
            view.into_tail().extract()
            // here, the view dies but the extracted reference lives on
        };

        assert_eq!(&data_region(1024, 0)[1..], extracted);
    }

    #[test]
    fn given_immutableview_with_reftovec_when_extractingimmutableref() {
        define_layout!(layout, LittleEndian, {
            field: u8,
            tail: [u8],
        });

        let storage = data_region(1024, 0);
        let extracted: &[u8] = {
            let view: layout::View<&Vec<u8>> = layout::View::new(&storage);
            view.into_tail().extract()
            // here, the view dies but the extracted reference lives on
        };

        assert_eq!(&data_region(1024, 0)[1..], extracted);
    }

    #[test]
    fn given_mutableview_when_extractingimmutableref() {
        define_layout!(layout, LittleEndian, {
            field: u8,
            tail: [u8],
        });

        let mut storage = data_region(1024, 0);
        let extracted: &[u8] = {
            let view: layout::View<&mut [u8]> = layout::View::new(&mut storage);
            view.into_tail().extract()
        };

        assert_eq!(&data_region(1024, 0)[1..], extracted);
    }

    #[test]
    fn given_mutableview_with_reftovec_when_extractingimmutableref() {
        define_layout!(layout, LittleEndian, {
            field: u8,
            tail: [u8],
        });

        let mut storage = data_region(1024, 0);
        let extracted: &[u8] = {
            let view: layout::View<&mut Vec<u8>> = layout::View::new(&mut storage);
            view.into_tail().extract()
        };

        assert_eq!(&data_region(1024, 0)[1..], extracted);
    }

    #[test]
    fn given_mutableview_when_extractingmutableref() {
        define_layout!(layout, LittleEndian, {
            field: u8,
            tail: [u8],
        });

        let mut storage = data_region(1024, 0);
        let extracted: &mut [u8] = {
            let view: layout::View<&mut [u8]> = layout::View::new(&mut storage);
            view.into_tail().extract()
        };

        assert_eq!(&data_region(1024, 0)[1..], extracted);
    }

    #[test]
    fn given_mutableview_with_reftovec_when_extractingmutableref() {
        define_layout!(layout, LittleEndian, {
            field: u8,
            tail: [u8],
        });

        let mut storage = data_region(1024, 0);
        let extracted: &mut [u8] = {
            let view: layout::View<&mut Vec<u8>> = layout::View::new(&mut storage);
            view.into_tail().extract()
        };

        assert_eq!(&data_region(1024, 0)[1..], extracted);
    }

    #[test]
    fn test_little_endian() {
        define_layout!(my_layout, LittleEndian, {
            field1: u16,
            field2: i64,
        });

        let mut storage = data_region(1024, 0);
        let mut view = my_layout::View::new(&mut storage);
        view.field1_mut().write(1000);
        assert_eq!(1000, view.field1().read());
        view.field2_mut().write(10i64.pow(15));
        assert_eq!(10i64.pow(15), view.field2().read());
        assert_eq!(
            1000,
            u16::from_le_bytes((&storage[0..2]).try_into().unwrap())
        );
        assert_eq!(
            10i64.pow(15),
            i64::from_le_bytes((&storage[2..10]).try_into().unwrap())
        );
    }

    #[test]
    fn test_big_endian() {
        define_layout!(my_layout, BigEndian, {
            field1: u16,
            field2: i64,
        });

        let mut storage = data_region(1024, 0);
        let mut view = my_layout::View::new(&mut storage);
        view.field1_mut().write(1000);
        assert_eq!(1000, view.field1().read());
        view.field2_mut().write(10i64.pow(15));
        assert_eq!(10i64.pow(15), view.field2().read());
        assert_eq!(
            1000,
            u16::from_be_bytes((&storage[0..2]).try_into().unwrap())
        );
        assert_eq!(
            10i64.pow(15),
            i64::from_be_bytes((&storage[2..10]).try_into().unwrap())
        );
    }

    // TODO Test View
    //   - based on &[u8], &mut [u8], Vec<u8> storage
    // TODO Test that there can be multiple views if they're readonly
}
