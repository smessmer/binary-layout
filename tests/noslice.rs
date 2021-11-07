use binary_layout::prelude::*;
use std::convert::TryInto;

mod common;
use common::data_region;

define_layout!(noslice, LittleEndian, {
    first: i8,
    second: i64,
    third: u16,
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
fn fields() {
    let mut storage = data_region(1024, 5);

    // Test initial data is read correctly
    assert_eq!(
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        noslice::first::read(&storage)
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        noslice::second::read(&storage)
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[9..11]).try_into().unwrap()),
        noslice::third::read(&storage)
    );

    // Test data can be written
    noslice::first::write(&mut storage, 60);
    noslice::second::write(&mut storage, -100_000_000_000);
    noslice::third::write(&mut storage, 1_000);

    // Test reading will return changed data
    assert_eq!(60, noslice::first::read(&storage));
    assert_eq!(-100_000_000_000, noslice::second::read(&storage));
    assert_eq!(1_000, noslice::third::read(&storage));
}

#[test]
fn view_readonly() {
    let storage = data_region(1024, 5);
    let view = noslice::View::new(&storage);

    // Test initial data is read correctly
    assert_eq!(
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[9..11]).try_into().unwrap()),
        view.third().read()
    );

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage();
    assert_eq!(extracted_storage, &storage);
}

#[test]
fn view_readwrite() {
    let mut storage = data_region(1024, 5);
    let mut view = noslice::View::new(&mut storage);

    // Test initial data is read correctly
    assert_eq!(
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[9..11]).try_into().unwrap()),
        view.third().read()
    );

    // Test data can be written
    view.first_mut().write(50);
    view.second_mut().write(10i64.pow(15));
    view.third_mut().write(1000);

    // Test reading will return changed data
    assert_eq!(50, view.first().read());
    assert_eq!(10i64.pow(15), view.second().read());
    assert_eq!(1000, view.third().read());

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
    let view = noslice::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[9..11]).try_into().unwrap()),
        view.third().read()
    );

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage();
    assert_eq!(&data_region(1024, 5), &extracted_storage);
}

#[test]
fn view_vec_readwrite() {
    let mut view = noslice::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[9..11]).try_into().unwrap()),
        view.third().read()
    );

    // Test data can be written
    view.first_mut().write(50);
    view.second_mut().write(10i64.pow(15));
    view.third_mut().write(1000);

    // Test reading will return changed data
    assert_eq!(50, view.first().read());
    assert_eq!(10i64.pow(15), view.second().read());
    assert_eq!(1000, view.third().read());

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
