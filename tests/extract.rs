use binary_layout::{prelude::*, Data};

mod common;
use common::data_region;

#[test]
fn given_immutableview_when_extractingimmutableref() {
    define_layout!(layout, LittleEndian, {
        field: u8,
        tail: [u8],
    });

    let storage = data_region(1024, 0);
    let extracted = {
        let view = layout::View::new(&storage);
        view.into_tail()
        // here, the view dies but the extracted reference lives on
    };

    assert_eq!(&data_region(1024, 0)[1..], &*extracted);
}

#[test]
fn given_immutableview_with_reftovec_when_extractingimmutableref() {
    define_layout!(layout, LittleEndian, {
        field: u8,
        tail: [u8],
    });

    let storage = data_region(1024, 0);
    let extracted: Data<&Vec<u8>> = {
        let view: layout::View<&Vec<u8>> = layout::View::new(&storage);
        view.into_tail()
        // here, the view dies but the extracted reference lives on
    };

    assert_eq!(&data_region(1024, 0)[1..], &*extracted);
}

#[test]
fn given_mutableview_when_extractingimmutableref() {
    define_layout!(layout, LittleEndian, {
        field: u8,
        tail: [u8],
    });

    let mut storage = data_region(1024, 0);
    let extracted: Data<&mut [u8]> = {
        let view: layout::View<&mut [u8]> = layout::View::new(&mut storage);
        view.into_tail()
    };

    assert_eq!(&data_region(1024, 0)[1..], &*extracted);
}

#[test]
fn given_mutableview_with_reftovec_when_extractingimmutableref() {
    define_layout!(layout, LittleEndian, {
        field: u8,
        tail: [u8],
    });

    let mut storage = data_region(1024, 0);
    let extracted: Data<&mut Vec<u8>> = {
        let view: layout::View<&mut Vec<u8>> = layout::View::new(&mut storage);
        view.into_tail()
    };

    assert_eq!(&data_region(1024, 0)[1..], &*extracted);
}

#[test]
fn given_mutableview_when_extractingmutableref() {
    define_layout!(layout, LittleEndian, {
        field: u8,
        tail: [u8],
    });

    let mut storage = data_region(1024, 0);
    let extracted: Data<&mut [u8]> = {
        let view: layout::View<&mut [u8]> = layout::View::new(&mut storage);
        view.into_tail()
    };

    assert_eq!(&data_region(1024, 0)[1..], &*extracted);
}

#[test]
fn given_mutableview_with_reftovec_when_extractingmutableref() {
    define_layout!(layout, LittleEndian, {
        field: u8,
        tail: [u8],
    });

    let mut storage = data_region(1024, 0);
    let extracted: Data<&mut Vec<u8>> = {
        let view: layout::View<&mut Vec<u8>> = layout::View::new(&mut storage);
        view.into_tail()
    };

    assert_eq!(&data_region(1024, 0)[1..], &*extracted);
}
