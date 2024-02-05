use binary_layout::prelude::*;
use std::convert::TryInto;
use std::num::NonZeroU128;

mod common;
use common::data_region;

define_layout!(deep_nesting, LittleEndian, {
    field1: u16,
});
define_layout!(header, BigEndian, {
    field1: i16,
});
define_layout!(middle, NativeEndian, {
    deep: deep_nesting::NestedView,
    field1: u16,
});
define_layout!(footer, BigEndian, {
    field1: u32,
    deep: deep_nesting::NestedView,
    tail: [u8],
});
define_layout!(whole, LittleEndian, {
    head: header::NestedView,
    field1: u64,
    mid: middle::NestedView,
    field2: NonZeroU128,
    foot: footer::NestedView,
});

#[test]
fn metadata() {
    assert_eq!(0, whole::head::OFFSET);
    assert_eq!(Some(2), whole::head::SIZE);
    assert_eq!(2, whole::field1::OFFSET);
    assert_eq!(Some(8), whole::field1::SIZE);
    assert_eq!(10, whole::mid::OFFSET);
    assert_eq!(Some(4), whole::mid::SIZE);
    assert_eq!(14, whole::field2::OFFSET);
    assert_eq!(Some(16), whole::field2::SIZE);
    assert_eq!(30, whole::foot::OFFSET);
    assert_eq!(None, whole::foot::SIZE);
}

// TODO Test field API once it supports nesting (see "fields" test function in other integration tests here)

#[test]
fn view_readonly() {
    let storage = data_region(1024, 5);
    let view = whole::View::new(&storage);

    // Test initial data is read correctly
    assert_eq!(
        i16::from_be_bytes((&data_region(1024, 5)[0..2]).try_into().unwrap()),
        view.head().field1().read(),
    );
    assert_eq!(
        u64::from_le_bytes((&data_region(1024, 5)[2..10]).try_into().unwrap()),
        view.field1().read(),
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[10..12]).try_into().unwrap()),
        view.mid().deep().field1().read(),
    );
    assert_eq!(
        u16::from_ne_bytes((&data_region(1024, 5)[12..14]).try_into().unwrap()),
        view.mid().field1().read(),
    );
    assert_eq!(
        u128::from_le_bytes((&data_region(1024, 5)[14..30]).try_into().unwrap()),
        view.field2().try_read().unwrap().get(),
    );
    assert_eq!(
        u32::from_be_bytes((&data_region(1024, 5)[30..34]).try_into().unwrap()),
        view.foot().field1().read()
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[34..36]).try_into().unwrap()),
        view.foot().deep().field1().read(),
    );
    assert_eq!(&data_region(1024, 5)[36..], view.foot().tail());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage();
    assert_eq!(&storage, extracted_storage);

    // Test into_storage on subfield will return correct data
    let view = whole::View::new(&storage);
    let extracted_storage = view.foot().into_storage();
    assert_eq!(&storage[30..], extracted_storage);
}

#[test]
fn view_readwrite() {
    let mut storage = data_region(1024, 5);
    let mut view = whole::View::new(&mut storage);

    // Test initial data is read correctly
    assert_eq!(
        i16::from_be_bytes((&data_region(1024, 5)[0..2]).try_into().unwrap()),
        view.head().field1().read(),
    );
    assert_eq!(
        u64::from_le_bytes((&data_region(1024, 5)[2..10]).try_into().unwrap()),
        view.field1().read(),
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[10..12]).try_into().unwrap()),
        view.mid().deep().field1().read(),
    );
    assert_eq!(
        u16::from_ne_bytes((&data_region(1024, 5)[12..14]).try_into().unwrap()),
        view.mid().field1().read(),
    );
    assert_eq!(
        u128::from_le_bytes((&data_region(1024, 5)[14..30]).try_into().unwrap()),
        view.field2().try_read().unwrap().get(),
    );
    assert_eq!(
        u32::from_be_bytes((&data_region(1024, 5)[30..34]).try_into().unwrap()),
        view.foot().field1().read()
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[34..36]).try_into().unwrap()),
        view.foot().deep().field1().read(),
    );
    assert_eq!(&data_region(1024, 5)[36..], view.foot().tail());

    // Test data can be written
    view.head_mut().field1_mut().write(-40);
    view.field1_mut().write(50);
    view.mid_mut().deep_mut().field1_mut().write(10);
    view.mid_mut().field1_mut().write(1000);
    view.field2_mut()
        .write(NonZeroU128::new(10_u128.pow(30)).unwrap());
    view.foot_mut().field1_mut().write(10_u32.pow(7));
    view.foot_mut().deep_mut().field1_mut().write(20);
    view.foot_mut()
        .tail_mut()
        .copy_from_slice(&data_region(1024, 6)[36..]);

    // Test reading will return changed data
    assert_eq!(-40, view.head().field1().read());
    assert_eq!(50, view.field1().read());
    assert_eq!(10, view.mid().deep().field1().read());
    assert_eq!(1000, view.mid().field1().read());
    assert_eq!(10_u128.pow(30), view.field2().try_read().unwrap().get());
    assert_eq!(10_u32.pow(7), view.foot().field1().read());
    assert_eq!(20, view.foot().deep().field1().read());
    assert_eq!(&data_region(1024, 6)[36..], view.foot().tail());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage().to_vec();
    assert_eq!(&storage, &extracted_storage);

    // Test into_storage on subfield will return correct data
    let view = whole::View::new(&storage);
    let foot = view.foot();
    let extracted_storage = foot.into_storage();
    assert_eq!(&storage[30..], extracted_storage);

    // Test storage is actually changed
    assert_eq!(
        -40,
        i16::from_be_bytes((&storage[0..2]).try_into().unwrap())
    );
    assert_eq!(
        50,
        u64::from_le_bytes((&storage[2..10]).try_into().unwrap())
    );
    assert_eq!(
        10,
        u16::from_le_bytes((&storage[10..12]).try_into().unwrap())
    );
    assert_eq!(
        1000,
        u16::from_ne_bytes((&storage[12..14]).try_into().unwrap())
    );
    assert_eq!(
        10_u128.pow(30),
        u128::from_le_bytes((&storage[14..30]).try_into().unwrap())
    );
    assert_eq!(
        10_u32.pow(7),
        u32::from_be_bytes((&storage[30..34]).try_into().unwrap())
    );
    assert_eq!(
        20,
        u16::from_le_bytes((&storage[34..36]).try_into().unwrap())
    );
    assert_eq!(&data_region(1024, 6)[36..], &storage[36..]);
}

#[test]
fn view_vec_readonly() {
    let view = whole::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(
        i16::from_be_bytes((&data_region(1024, 5)[0..2]).try_into().unwrap()),
        view.head().field1().read(),
    );
    assert_eq!(
        u64::from_le_bytes((&data_region(1024, 5)[2..10]).try_into().unwrap()),
        view.field1().read(),
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[10..12]).try_into().unwrap()),
        view.mid().deep().field1().read(),
    );
    assert_eq!(
        u16::from_ne_bytes((&data_region(1024, 5)[12..14]).try_into().unwrap()),
        view.mid().field1().read(),
    );
    assert_eq!(
        u128::from_le_bytes((&data_region(1024, 5)[14..30]).try_into().unwrap()),
        view.field2().try_read().unwrap().get(),
    );
    assert_eq!(
        u32::from_be_bytes((&data_region(1024, 5)[30..34]).try_into().unwrap()),
        view.foot().field1().read()
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[34..36]).try_into().unwrap()),
        view.foot().deep().field1().read(),
    );
    assert_eq!(&data_region(1024, 5)[36..], view.foot().tail());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage();
    assert_eq!(&data_region(1024, 5), &*extracted_storage);

    // Test into_storage on subfield will return correct data
    let view = whole::View::new(data_region(1024, 5));
    let foot = view.into_foot();
    let extracted_storage = foot.into_storage();
    assert_eq!(&data_region(1024, 5)[30..], extracted_storage.as_ref());
}

#[test]
fn view_vec_readwrite() {
    let mut view = whole::View::new(data_region(1024, 5));

    // Test initial data is read correctly
    assert_eq!(
        i16::from_be_bytes((&data_region(1024, 5)[0..2]).try_into().unwrap()),
        view.head().field1().read(),
    );
    assert_eq!(
        u64::from_le_bytes((&data_region(1024, 5)[2..10]).try_into().unwrap()),
        view.field1().read(),
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[10..12]).try_into().unwrap()),
        view.mid().deep().field1().read(),
    );
    assert_eq!(
        u16::from_ne_bytes((&data_region(1024, 5)[12..14]).try_into().unwrap()),
        view.mid().field1().read(),
    );
    assert_eq!(
        u128::from_le_bytes((&data_region(1024, 5)[14..30]).try_into().unwrap()),
        view.field2().try_read().unwrap().get(),
    );
    assert_eq!(
        u32::from_be_bytes((&data_region(1024, 5)[30..34]).try_into().unwrap()),
        view.foot().field1().read()
    );
    assert_eq!(
        u16::from_le_bytes((&data_region(1024, 5)[34..36]).try_into().unwrap()),
        view.foot().deep().field1().read(),
    );
    assert_eq!(&data_region(1024, 5)[36..], view.foot().tail());

    // Test data can be written
    let mutate = |view: &mut whole::View<Vec<u8>>| {
        view.head_mut().field1_mut().write(-40);
        view.field1_mut().write(50);
        view.mid_mut().deep_mut().field1_mut().write(10);
        view.mid_mut().field1_mut().write(1000);
        view.field2_mut()
            .write(NonZeroU128::new(10_u128.pow(30)).unwrap());
        view.foot_mut().field1_mut().write(10_u32.pow(7));
        view.foot_mut().deep_mut().field1_mut().write(20);
        view.foot_mut()
            .tail_mut()
            .copy_from_slice(&data_region(1024, 6)[36..]);
    };
    mutate(&mut view);

    // Test reading will return changed data
    assert_eq!(-40, view.head().field1().read());
    assert_eq!(50, view.field1().read());
    assert_eq!(10, view.mid().deep().field1().read());
    assert_eq!(1000, view.mid().field1().read());
    assert_eq!(10_u128.pow(30), view.field2().try_read().unwrap().get());
    assert_eq!(10_u32.pow(7), view.foot().field1().read());
    assert_eq!(20, view.foot().deep().field1().read());
    assert_eq!(&data_region(1024, 6)[36..], view.foot().tail());

    // Test into_storage will return correct data
    let extracted_storage = view.into_storage();
    assert_eq!(
        -40,
        i16::from_be_bytes((&extracted_storage[0..2]).try_into().unwrap())
    );
    assert_eq!(
        50,
        u64::from_le_bytes((&extracted_storage[2..10]).try_into().unwrap())
    );
    assert_eq!(
        10,
        u16::from_le_bytes((&extracted_storage[10..12]).try_into().unwrap())
    );
    assert_eq!(
        1000,
        u16::from_ne_bytes((&extracted_storage[12..14]).try_into().unwrap())
    );
    assert_eq!(
        10_u128.pow(30),
        u128::from_le_bytes((&extracted_storage[14..30]).try_into().unwrap())
    );
    assert_eq!(
        10_u32.pow(7),
        u32::from_be_bytes((&extracted_storage[30..34]).try_into().unwrap())
    );
    assert_eq!(
        20,
        u16::from_le_bytes((&extracted_storage[34..36]).try_into().unwrap())
    );
    assert_eq!(&data_region(1024, 6)[36..], &extracted_storage[36..]);

    // Test into_storage on subfield will return correct data
    let mut view = whole::View::new(data_region(1024, 5));
    mutate(&mut view);
    let extracted_storage_2 = view.into_foot().into_storage();
    assert_eq!(&&extracted_storage[30..], &extracted_storage_2.as_ref());
}
