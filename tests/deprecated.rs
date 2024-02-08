#![allow(deprecated)]

use core::num::NonZeroI32;

use binary_layout::define_layout;

// Test that the deprecated `define_layout` macro is still available
define_layout!(withslice, LittleEndian, {
    first: i8,
    second: i64,
    third: [u8; 5],
    fourth: u16,
    fifth: NonZeroI32,
    sixth: [u8],
});
