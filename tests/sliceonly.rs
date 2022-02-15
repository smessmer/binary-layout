use binary_layout::prelude::*;

mod common;
use common::data_region;

define_layout!(sliceonly, LittleEndian, { field: [u8] });

#[test]
fn metadata() {
    assert_eq!(0, sliceonly::field::OFFSET);
    assert_eq!(None, sliceonly::field::SIZE);
}

#[test]
fn fields() {
    let mut storage = data_region(1024, 5);

    // Test initial data is read correctly
    assert_eq!(&data_region(1024, 5), sliceonly::field::data(&storage));

    // Test data can be written
    sliceonly::field::data_mut(&mut storage).copy_from_slice(&data_region(1024, 6));

    // Test reading will return changed data
    assert_eq!(&data_region(1024, 6), sliceonly::field::data(&storage));
}

#[test]
fn view_readonly() {
    let storage = data_region(1024, 5);
    let view = sliceonly::View::new(&storage);

    // Test initial data is read correctly
    assert_eq!(&data_region(1024, 5), view.field());

    // Test into_storage will return correct data
    let extracted_storage: &Vec<u8> = view.into_storage();
    assert_eq!(*extracted_storage, storage);
}

#[test]
fn view_readwrite() {
    let mut storage = data_region(1024, 5);
    let mut view = sliceonly::View::new(&mut storage);

    // Test initial data is read correctly
    assert_eq!(&data_region(1024, 5), view.field());

    // Test data can be written
    view.field_mut().copy_from_slice(&data_region(1024, 6));

    // Test reading will return changed data
    assert_eq!(&data_region(1024, 6), view.field());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage().to_vec();
    assert_eq!(&storage, &extracted_storage);

    // Test original storage is changed
    assert_eq!(&data_region(1024, 6), &storage);
}

#[test]
fn view_vec_readonly() {
    let view = sliceonly::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(&data_region(1024, 5), view.field());

    // Test into_storage will return correct data
    let storage: Vec<u8> = view.into_storage();
    assert_eq!(&data_region(1024, 5), &*storage);
}

#[test]
fn view_vec_readwrite() {
    let mut view = sliceonly::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(&data_region(1024, 5), view.field());

    // Test data can be written
    view.field_mut().copy_from_slice(&data_region(1024, 6));

    // Test reading will return changed data
    assert_eq!(&data_region(1024, 6), view.field());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage();
    assert_eq!(&data_region(1024, 6), &*extracted_storage);
}
