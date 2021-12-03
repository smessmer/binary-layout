use binary_layout::prelude::*;
use std::convert::TryInto;

mod common;
use common::data_region;

define_layout!(withslice, LittleEndian, {
    first: i8,
    second: i64,
    third: [u8; 5],
    fourth: u16,
    fifth: [u8],
});

#[test]
fn metadata() {
    assert_eq!(0, withslice::first::OFFSET);
    assert_eq!(Some(1), withslice::first::SIZE);
    assert_eq!(1, withslice::second::OFFSET);
    assert_eq!(Some(8), withslice::second::SIZE);
    assert_eq!(9, withslice::third::OFFSET);
    assert_eq!(Some(5), withslice::third::SIZE);
    assert_eq!(14, withslice::fourth::OFFSET);
    assert_eq!(Some(2), withslice::fourth::SIZE);
    assert_eq!(16, withslice::fifth::OFFSET);
    assert_eq!(None, withslice::fifth::SIZE);
}

#[test]
fn fields() {
    let mut storage = data_region(1024, 5);

    // Test initial data is read correctly
    assert_eq!(5, withslice::third::data(&storage).len());
    assert_eq!(5, withslice::third::data_mut(&mut storage).len());
    assert_eq!(1024 - 16, withslice::fifth::data(&storage).len());
    assert_eq!(1024 - 16, withslice::fifth::data_mut(&mut storage).len());

    // Test data can be written
    withslice::first::write(&mut storage, 60);
    withslice::second::write(&mut storage, -100_000_000_000);
    withslice::third::data_mut(&mut storage).copy_from_slice(&[10, 20, 30, 40, 50]);
    withslice::fourth::write(&mut storage, 1_000);
    withslice::fifth::data_mut(&mut storage).copy_from_slice(&data_region(1024 - 16, 6));

    // Test reading will return changed data
    assert_eq!(60, withslice::first::read(&storage));
    assert_eq!(-100_000_000_000, withslice::second::read(&storage));
    assert_eq!(&[10, 20, 30, 40, 50], withslice::third::data(&storage));
    assert_eq!(1_000, withslice::fourth::read(&storage));
    assert_eq!(&data_region(1024 - 16, 6), withslice::fifth::data(&storage));
}

#[test]
fn view_readonly() {
    let storage = data_region(1024, 5);
    let view = withslice::View::new(&storage);

    // Test initial data is read correctly
    assert_eq!(
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(&data_region(1024, 5)[9..14], view.third(),);
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[14..16]).try_into().unwrap()),
        view.fourth().read()
    );
    assert_eq!(&data_region(1024, 5)[16..], view.fifth());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage();
    assert_eq!(&storage, &*extracted_storage);
}

#[test]
fn view_readwrite() {
    let mut storage = data_region(1024, 5);
    let mut view = withslice::View::new(&mut storage);

    // Test initial data is read correctly
    assert_eq!(
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(&data_region(1024, 5)[9..14], view.third(),);
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[14..16]).try_into().unwrap()),
        view.fourth().read()
    );
    assert_eq!(&data_region(1024, 5)[16..], view.fifth());

    // Test data can be written
    view.first_mut().write(50);
    view.second_mut().write(10i64.pow(15));
    view.third_mut().copy_from_slice(&[10, 20, 30, 40, 50]);
    view.fourth_mut().write(1000);
    view.fifth_mut()
        .copy_from_slice(&data_region(1024, 6)[16..]);

    // Test reading will return changed data
    assert_eq!(50, view.first().read());
    assert_eq!(10i64.pow(15), view.second().read());
    assert_eq!(&[10, 20, 30, 40, 50], view.third());
    assert_eq!(1000, view.fourth().read());
    assert_eq!(&data_region(1024, 6)[16..], view.fifth());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage().to_vec();
    assert_eq!(&storage, &extracted_storage);

    // Test storage is actually changed
    assert_eq!(50, i8::from_le_bytes((&storage[0..1]).try_into().unwrap()));
    assert_eq!(
        10i64.pow(15),
        i64::from_le_bytes((&storage[1..9]).try_into().unwrap())
    );
    assert_eq!(&[10, 20, 30, 40, 50], &storage[9..14]);
    assert_eq!(
        1000,
        u16::from_le_bytes((&storage[14..16]).try_into().unwrap())
    );
    assert_eq!(&data_region(1024, 6)[16..], &storage[16..]);
}

#[test]
fn view_vec_readonly() {
    let view = withslice::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(&data_region(1024, 5)[9..14], view.third(),);
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[14..16]).try_into().unwrap()),
        view.fourth().read()
    );
    assert_eq!(&data_region(1024, 5)[16..], view.fifth());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage();
    assert_eq!(&data_region(1024, 5), &*extracted_storage);
}

#[test]
fn view_vec_readwrite() {
    let mut view = withslice::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(
        i8::from_le_bytes((&data_region(1024, 5)[0..1]).try_into().unwrap()),
        view.first().read()
    );
    assert_eq!(
        i64::from_le_bytes((&data_region(1024, 5)[1..9]).try_into().unwrap()),
        view.second().read()
    );
    assert_eq!(&data_region(1024, 5)[9..14], view.third(),);
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[14..16]).try_into().unwrap()),
        view.fourth().read()
    );
    assert_eq!(&data_region(1024, 5)[16..], view.fifth());

    // Test data can be written
    view.first_mut().write(50);
    view.second_mut().write(10i64.pow(15));
    view.third_mut().copy_from_slice(&[10, 20, 30, 40, 50]);
    view.fourth_mut().write(1000);
    view.fifth_mut()
        .copy_from_slice(&data_region(1024, 6)[16..]);

    // Test reading will return changed data
    assert_eq!(50, view.first().read());
    assert_eq!(10i64.pow(15), view.second().read());
    assert_eq!(&[10, 20, 30, 40, 50], view.third());
    assert_eq!(1000, view.fourth().read());
    assert_eq!(&data_region(1024, 6)[16..], view.fifth());

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
    assert_eq!(&[10, 20, 30, 40, 50], &extracted_storage[9..14]);
    assert_eq!(
        1000,
        u16::from_le_bytes((&extracted_storage[14..16]).try_into().unwrap())
    );
    assert_eq!(&data_region(1024, 6)[16..], &extracted_storage[16..]);
}
