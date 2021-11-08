use binary_layout::prelude::*;
use std::convert::TryInto;

mod common;
use common::data_region;

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
