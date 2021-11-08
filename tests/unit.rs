use binary_layout::prelude::*;

#[test]
fn metadata() {
    define_layout!(unit_layout_1, LittleEndian, { field1: () });

    assert_eq!(0, unit_layout_1::field1::OFFSET);
    assert_eq!(Some(0), unit_layout_1::field1::SIZE);

    define_layout!(unit_layout_2, LittleEndian, {
        field1: u8,
        field2: (),
        field3: u128
    });

    assert_eq!(0, unit_layout_2::field1::OFFSET);
    assert_eq!(Some(1), unit_layout_2::field1::SIZE);
    assert_eq!(1, unit_layout_2::field2::OFFSET);
    assert_eq!(Some(0), unit_layout_2::field2::SIZE);
    assert_eq!(1, unit_layout_2::field3::OFFSET);
    assert_eq!(Some(16), unit_layout_2::field3::SIZE);
}

#[test]
fn test_layout_with_unit() {
    let mut storage: [u8; 1024] = [0; 1024];

    define_layout!(unit_layout_1, LittleEndian, { field1: () });
    let mut view = unit_layout_1::View::new(&mut storage[0..0]); // Zero length slice
    view.field1_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage, [0u8; 1024]);
    storage.fill(0u8);

    define_layout!(unit_layout_2, LittleEndian, {
        field1: (),
        field2: ()
    });
    let mut view = unit_layout_2::View::new(&mut storage[0..0]); // Zero length slice
    view.field1_mut().write(()); // Shouldn't cause any issues.
    view.field2_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage, [0u8; 1024]);
    storage.fill(0u8);

    define_layout!(unit_layout_3, LittleEndian, {
        field1: (),
        field2: (),
        field3: ()
    });
    let mut view = unit_layout_3::View::new(&mut storage[0..0]); // Zero length slice
    view.field1_mut().write(()); // Shouldn't cause any issues.
    view.field2_mut().write(()); // Shouldn't cause any issues.
    view.field3_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage, [0u8; 1024]);
    storage.fill(0u8);

    define_layout!(unit_layout_4, LittleEndian, {
        field1: u8,
        field2: ()
    });
    let mut view = unit_layout_4::View::new(&mut storage[0..1]);
    view.field1_mut().write(3u8); // Shouldn't cause any issues.
    view.field2_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage[0], 3u8);
    assert_eq!(storage[1..], [0u8; 1023]);
    storage.fill(0u8);

    define_layout!(unit_layout_5, LittleEndian, {
        field1: (),
        field2: u8
    });
    let mut view = unit_layout_5::View::new(&mut storage[0..1]);
    view.field1_mut().write(()); // Shouldn't cause any issues.
    view.field2_mut().write(5u8); // Shouldn't cause any issues.
    assert_eq!(storage[0], 5u8);
    assert_eq!(storage[1..], [0u8; 1023]);
    storage.fill(0u8);

    define_layout!(unit_layout_6, LittleEndian, {
        field1: (),
        field2: u8,
        field3: ()
    });
    let mut view = unit_layout_6::View::new(&mut storage[0..1]);
    view.field1_mut().write(()); // Shouldn't cause any issues.
    view.field2_mut().write(37u8); // Shouldn't cause any issues.
    view.field3_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage[0], 37u8);
    assert_eq!(storage[1..], [0u8; 1023]);
    storage.fill(0u8);

    define_layout!(unit_layout_7, LittleEndian, {
        field1: u8,
        field2: (),
        field3: u128
    });
    let mut view = unit_layout_7::View::new(&mut storage);
    view.field1_mut().write(43u8); // Shouldn't cause any issues.
    view.field2_mut().write(()); // Shouldn't cause any issues.
    view.field3_mut().write(120u128); // Shouldn't cause any issues.
    assert_eq!(storage[0], 43u8);
    assert_eq!(storage[1..17], (120u128).to_le_bytes());
    assert_eq!(storage[18..], [0u8; 1006]);
    storage.fill(0u8);

    define_layout!(unit_layout_8, BigEndian, { field1: () });
    let mut view = unit_layout_8::View::new(&mut storage[0..0]); // Zero length slice
    view.field1_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage, [0u8; 1024]);
    storage.fill(0u8);

    define_layout!(unit_layout_9, BigEndian, {
        field1: (),
        field2: ()
    });
    let mut view = unit_layout_9::View::new(&mut storage[0..0]); // Zero length slice
    view.field1_mut().write(()); // Shouldn't cause any issues.
    view.field2_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage, [0u8; 1024]);
    storage.fill(0u8);

    define_layout!(unit_layout_10, BigEndian, {
        field1: (),
        field2: (),
        field3: ()
    });
    let mut view = unit_layout_10::View::new(&mut storage[0..0]); // Zero length slice
    view.field1_mut().write(()); // Shouldn't cause any issues.
    view.field2_mut().write(()); // Shouldn't cause any issues.
    view.field3_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage, [0u8; 1024]);
    storage.fill(0u8);

    define_layout!(unit_layout_11, BigEndian, {
        field1: u8,
        field2: ()
    });
    let mut view = unit_layout_11::View::new(&mut storage[0..1]);
    view.field1_mut().write(3u8); // Shouldn't cause any issues.
    view.field2_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage[0], 3u8);
    assert_eq!(storage[1..], [0u8; 1023]);
    storage.fill(0u8);

    define_layout!(unit_layout_12, BigEndian, {
        field1: (),
        field2: u8
    });
    let mut view = unit_layout_12::View::new(&mut storage[0..1]);
    view.field1_mut().write(()); // Shouldn't cause any issues.
    view.field2_mut().write(5u8); // Shouldn't cause any issues.
    assert_eq!(storage[0], 5u8);
    assert_eq!(storage[1..], [0u8; 1023]);
    storage.fill(0u8);

    define_layout!(unit_layout_13, BigEndian, {
        field1: (),
        field2: u8,
        field3: ()
    });
    let mut view = unit_layout_13::View::new(&mut storage[0..1]);
    view.field1_mut().write(()); // Shouldn't cause any issues.
    view.field2_mut().write(37u8); // Shouldn't cause any issues.
    view.field3_mut().write(()); // Shouldn't cause any issues.
    assert_eq!(storage[0], 37u8);
    assert_eq!(storage[1..], [0u8; 1023]);
    storage.fill(0u8);

    define_layout!(unit_layout_14, BigEndian, {
        field1: u8,
        field2: (),
        field3: u128
    });
    let mut view = unit_layout_14::View::new(&mut storage);
    view.field1_mut().write(43u8); // Shouldn't cause any issues.
    view.field2_mut().write(()); // Shouldn't cause any issues.
    view.field3_mut().write(120u128); // Shouldn't cause any issues.
    assert_eq!(storage[0], 43u8);
    assert_eq!(storage[1..17], (120u128).to_be_bytes());
    assert_eq!(storage[18..], [0u8; 1006]);
    storage.fill(0u8);
}
