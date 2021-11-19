use binary_layout::prelude::*;

mod common;
use common::data_region;

#[test]
fn given_immutableview_when_extractingimmutableref() {
    define_layout!(layout, LittleEndian, {
        field: u8,
        tail: [u8],
    });

    let storage = data_region(1024, 0);
    let extracted: &[u8] = {
        let view = layout::View::new(&storage);
        view.into_tail().into_data()
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
        view.into_tail().into_data()
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
        view.into_tail().into_data()
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
        view.into_tail().into_data()
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
        view.into_tail().into_data()
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
        view.into_tail().into_data()
    };

    assert_eq!(&data_region(1024, 0)[1..], extracted);
}
