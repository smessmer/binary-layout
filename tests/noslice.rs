use binary_layout::prelude::*;
use std::convert::TryInto;
use std::num::NonZeroI32;

mod common;
use common::data_region;

binary_layout!(noslice, LittleEndian, {
    first: i8,
    second: i64,
    third: NonZeroI32,
    fourth: u16,
});

#[test]
fn metadata() {
    assert_eq!(0, noslice::first::OFFSET);
    assert_eq!(Some(1), noslice::first::SIZE);
    assert_eq!(1, noslice::second::OFFSET);
    assert_eq!(Some(8), noslice::second::SIZE);
    assert_eq!(9, noslice::third::OFFSET);
    assert_eq!(Some(4), noslice::third::SIZE);
    assert_eq!(13, noslice::fourth::OFFSET);
    assert_eq!(Some(2), noslice::fourth::SIZE);
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
        i32::from_le_bytes((&data_region(1024, 5)[9..13]).try_into().unwrap()),
        noslice::third::try_read(&storage).unwrap().get(),
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[13..15]).try_into().unwrap()),
        noslice::fourth::read(&storage)
    );

    // Test data can be written
    noslice::first::write(&mut storage, 60);
    noslice::second::write(&mut storage, -100_000_000_000);
    noslice::third::write(&mut storage, NonZeroI32::new(-1_000_000_000).unwrap());
    noslice::fourth::write(&mut storage, 1_000);

    // Test reading will return changed data
    assert_eq!(60, noslice::first::read(&storage));
    assert_eq!(-100_000_000_000, noslice::second::read(&storage));
    assert_eq!(
        -1_000_000_000,
        noslice::third::try_read(&storage).unwrap().get()
    );
    assert_eq!(1_000, noslice::fourth::read(&storage));
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
        i32::from_le_bytes((&data_region(1024, 5)[9..13]).try_into().unwrap()),
        view.third().try_read().unwrap().get(),
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[13..15]).try_into().unwrap()),
        view.fourth().read()
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
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(
        i32::from_le_bytes((&data_region(1024, 5)[9..13]).try_into().unwrap()),
        view.third().try_read().unwrap().get(),
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[13..15]).try_into().unwrap()),
        view.fourth().read()
    );

    // Test data can be written
    view.first_mut().write(50);
    view.second_mut().write(10i64.pow(15));
    view.third_mut()
        .write(NonZeroI32::new(10i32.pow(8)).unwrap());
    view.fourth_mut().write(1000);

    // Test reading will return changed data
    assert_eq!(50, view.first().read());
    assert_eq!(10i64.pow(15), view.second().read());
    assert_eq!(
        NonZeroI32::new(10i32.pow(8)).unwrap(),
        view.third().try_read().unwrap()
    );
    assert_eq!(1000, view.fourth().read());

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
        10i32.pow(8),
        i32::from_le_bytes((&storage[9..13]).try_into().unwrap())
    );
    assert_eq!(
        1000,
        u16::from_le_bytes((&storage[13..15]).try_into().unwrap())
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
        i32::from_le_bytes((&data_region(1024, 5)[9..13]).try_into().unwrap()),
        view.third().try_read().unwrap().get(),
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[13..15]).try_into().unwrap()),
        view.fourth().read()
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
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(
        i32::from_le_bytes((&data_region(1024, 5)[9..13]).try_into().unwrap()),
        view.third().try_read().unwrap().get(),
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[13..15]).try_into().unwrap()),
        view.fourth().read()
    );

    // Test data can be written
    view.first_mut().write(50);
    view.second_mut().write(10i64.pow(15));
    view.third_mut()
        .write(NonZeroI32::new(10i32.pow(8)).unwrap());
    view.fourth_mut().write(1000);

    // Test reading will return changed data
    assert_eq!(50, view.first().read());
    assert_eq!(10i64.pow(15), view.second().read());
    assert_eq!(
        NonZeroI32::new(10i32.pow(8)).unwrap(),
        view.third().try_read().unwrap()
    );
    assert_eq!(1000, view.fourth().read());

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
        10i32.pow(8),
        i32::from_le_bytes((&extracted_storage[9..13]).try_into().unwrap())
    );
    assert_eq!(
        1000,
        u16::from_le_bytes((&extracted_storage[13..15]).try_into().unwrap())
    );
}
