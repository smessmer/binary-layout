use binary_layout::prelude::*;

#[test]
fn test_layout_empty() {
    define_layout!(empty_little, LittleEndian, {});
    define_layout!(empty_big, BigEndian, {});
    define_layout!(empty_native, NativeEndian, {});
}
