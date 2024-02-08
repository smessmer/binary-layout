use binary_layout::prelude::*;

#[test]
fn test_layout_empty() {
    binary_layout!(empty_little, LittleEndian, {});
    binary_layout!(empty_big, BigEndian, {});
    binary_layout!(empty_native, NativeEndian, {});
}
