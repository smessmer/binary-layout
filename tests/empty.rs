use binary_layout::prelude::*;

#[test]
fn test_layout_empty() {
    define_layout!(empty, LittleEndian, {});
}
