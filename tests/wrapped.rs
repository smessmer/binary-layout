use binary_layout::{prelude::*, LayoutAs};
use core::any::{Any, TypeId};
use core::convert::TryInto;
use std::convert::Infallible;

mod common;
use common::data_region;

#[derive(Debug, PartialEq, Eq)]
pub struct Wrapped<T>(T);
impl<T> LayoutAs<T> for Wrapped<T> {
    type ReadError = Infallible;
    type WriteError = Infallible;

    fn try_read(v: T) -> Result<Wrapped<T>, Infallible> {
        Ok(Wrapped(v))
    }

    fn try_write(v: Wrapped<T>) -> Result<T, Infallible> {
        Ok(v.0)
    }
}

define_layout!(noslice, LittleEndian, {
    first: Wrapped<i8> as i8,
    second: Wrapped<i64> as i64,
    third: Wrapped<u16> as u16,
    // TODO Can we allow wrapping NonZero types?
});

#[test]
fn metadata() {
    assert_eq!(0, noslice::first::OFFSET);
    assert_eq!(Some(1), noslice::first::SIZE);
    assert_eq!(1, noslice::second::OFFSET);
    assert_eq!(Some(8), noslice::second::SIZE);
    assert_eq!(9, noslice::third::OFFSET);
    assert_eq!(Some(2), noslice::third::SIZE);
}

#[test]
fn types() {
    let storage = data_region(1024, 5);
    let view = noslice::View::new(&storage);

    assert_eq!(
        TypeId::of::<Wrapped<i8>>(),
        noslice::first::read(&storage).type_id()
    );
    assert_eq!(
        TypeId::of::<Wrapped<i64>>(),
        noslice::second::read(&storage).type_id()
    );
    assert_eq!(
        TypeId::of::<Wrapped<u16>>(),
        noslice::third::read(&storage).type_id()
    );

    assert_eq!(TypeId::of::<Wrapped<i8>>(), view.first().read().type_id());
    assert_eq!(TypeId::of::<Wrapped<i64>>(), view.second().read().type_id());
    assert_eq!(TypeId::of::<Wrapped<u16>>(), view.third().read().type_id());
}

#[test]
fn fields() {
    let mut storage = data_region(1024, 5);

    // Test initial data is read correctly
    assert_eq!(
        Wrapped(i8::from_le_bytes(
            (&data_region(1024, 5)[0..1]).try_into().unwrap()
        )),
        noslice::first::read(&storage)
    );
    assert_eq!(
        Wrapped(i64::from_le_bytes(
            (&data_region(1024, 5)[1..9]).try_into().unwrap()
        )),
        noslice::second::read(&storage)
    );
    assert_eq!(
        Wrapped(u16::from_le_bytes(
            (&data_region(1024, 5)[9..11]).try_into().unwrap()
        )),
        noslice::third::read(&storage)
    );

    // Test data can be written
    noslice::first::write(&mut storage, Wrapped(60));
    noslice::second::write(&mut storage, Wrapped(-100_000_000_000));
    noslice::third::write(&mut storage, Wrapped(1_000));

    // Test reading will return changed data
    assert_eq!(Wrapped(60), noslice::first::read(&storage));
    assert_eq!(Wrapped(-100_000_000_000), noslice::second::read(&storage));
    assert_eq!(Wrapped(1_000), noslice::third::read(&storage));
}

#[test]
fn view_readonly() {
    let storage = data_region(1024, 5);
    let view = noslice::View::new(&storage);

    // Test initial data is read correctly
    assert_eq!(
        Wrapped(i8::from_le_bytes(
            (&data_region(1024, 5)[0..1]).try_into().unwrap()
        )),
        view.first().read()
    );
    assert_eq!(
        Wrapped(i64::from_le_bytes(
            (&data_region(1024, 5)[1..9]).try_into().unwrap()
        )),
        view.second().read()
    );
    assert_eq!(
        Wrapped(u16::from_le_bytes(
            (&data_region(1024, 5)[9..11]).try_into().unwrap()
        )),
        view.third().read()
    );

    // Test into_storage will return correct data
    let extracted_storage: &Vec<u8> = view.into_storage();
    assert_eq!(*extracted_storage, storage);
}

#[test]
fn view_readwrite() {
    let mut storage = data_region(1024, 5);
    let mut view = noslice::View::new(&mut storage);

    // Test initial data is read correctly
    assert_eq!(
        Wrapped(i8::from_le_bytes(
            (&data_region(1024, 5)[0..1]).try_into().unwrap()
        )),
        view.first().read()
    );
    assert_eq!(
        Wrapped(i64::from_le_bytes(
            (&data_region(1024, 5)[1..9]).try_into().unwrap()
        )),
        view.second().read()
    );
    assert_eq!(
        Wrapped(u16::from_le_bytes(
            (&data_region(1024, 5)[9..11]).try_into().unwrap()
        )),
        view.third().read()
    );

    // Test data can be written
    view.first_mut().write(Wrapped(50));
    view.second_mut().write(Wrapped(10i64.pow(15)));
    view.third_mut().write(Wrapped(1000));

    // Test reading will return changed data
    assert_eq!(Wrapped(50), view.first().read());
    assert_eq!(Wrapped(10i64.pow(15)), view.second().read());
    assert_eq!(Wrapped(1000), view.third().read());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage().to_vec();
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
    let view = noslice::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(
        Wrapped(i8::from_le_bytes(
            (&data_region(1024, 5)[0..1]).try_into().unwrap()
        )),
        view.first().read()
    );
    assert_eq!(
        Wrapped(i64::from_le_bytes(
            (&data_region(1024, 5)[1..9]).try_into().unwrap()
        )),
        view.second().read()
    );
    assert_eq!(
        Wrapped(u16::from_le_bytes(
            (&data_region(1024, 5)[9..11]).try_into().unwrap()
        )),
        view.third().read()
    );

    // Test into_storage will return correct data
    let extracted_storage: Vec<u8> = view.into_storage();
    assert_eq!(&data_region(1024, 5), &*extracted_storage);
}

#[test]
fn view_vec_readwrite() {
    let mut view = noslice::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(
        Wrapped(i8::from_le_bytes(
            (&data_region(1024, 5)[0..1]).try_into().unwrap()
        )),
        view.first().read()
    );
    assert_eq!(
        Wrapped(i64::from_le_bytes(
            (&data_region(1024, 5)[1..9]).try_into().unwrap()
        )),
        view.second().read()
    );
    assert_eq!(
        Wrapped(u16::from_le_bytes(
            (&data_region(1024, 5)[9..11]).try_into().unwrap()
        )),
        view.third().read()
    );

    // Test data can be written
    view.first_mut().write(Wrapped(50));
    view.second_mut().write(Wrapped(10i64.pow(15)));
    view.third_mut().write(Wrapped(1000));

    // Test reading will return changed data
    assert_eq!(Wrapped(50), view.first().read());
    assert_eq!(Wrapped(10i64.pow(15)), view.second().read());
    assert_eq!(Wrapped(1000), view.third().read());

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
