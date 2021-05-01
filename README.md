[![Build Status](https://github.com/smessmer/binary-layout/actions/workflows/ci.yml/badge.svg)](https://github.com/smessmer/binary-layout/actions/workflows/ci.yml)
[![Latest Version](https://img.shields.io/crates/v/binary-layout.svg)](https://crates.io/crates/binary-layout)
[![docs.rs](https://docs.rs/binary-layout/badge.svg)](https://docs.rs/binary-layout)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/smessmer/binary-layout/blob/master/LICENSE-MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://github.com/smessmer/binary-layout/blob/master/LICENSE-APACHE)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

# binary-layout

The [binary-layout](https://crates.io/crates/binary-layout) library allows type-safe, inplace, zero-copy access to structured binary data.
You define a custom data layout and give it a slice of binary data, and it will allow you to read and
write the fields defined in the layout from the binary data without having to copy any of the data.
It's similar to transmuting to/from a `#[repr(packed)]` struct, but [much safer](#why-not-reprpacked).

Note that the data does not go through serialization/deserialization or a parsing step.
All accessors access the underlying package data directly.

## Example
```rust
use binary_layout::prelude::*;

// See https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol for ICMP package layout
define_layout!(icmp_packet, BigEndian, {
  packet_type: u8,
  code: u8,
  checksum: u16,
  rest_of_header: [u8; 4],
  data_section: [u8], // open ended byte array, matches until the end of the package
});

fn func(packet_data: &mut [u8]) {
  let mut view = icmp_packet::View::new(packet_data);

  // read some data
  let code: u8 = view.code().read();
  // equivalent: let code: u8 = packet_data[1];

  // write some data
  view.checksum_mut().write(10);
  // equivalent: packet_data[2..4].copy_from_slice(&10u16.to_be_bytes());

  // access an open ended byte array
  let data_section: &[u8] = view.data_section().data();
  // equivalent: let data_section: &[u8] = &packet_data[8..];

  // and modify it
  view.data_section_mut().data_mut()[..5].copy_from_slice(&[1, 2, 3, 4, 5]);
  // equivalent: packet_data[8..13].copy_from_slice(&[1, 2, 3, 4, 5]);
}
```

## What to use this library for?
Anything that needs inplace zero-copy access to structured binary data.
- Network packets are an obvious example
- File system inodes
- Structured binary data in files if you want to avoid explicit (de)serialization, possibly in combination with [memmap](https://docs.rs/memmap).

## Why use this library?
- Inplace, zero-copy, type-safe access to your data.
- Data layout is defined in one central place, call sites can't accidentally use wrong field offsets.
- Convenient and simple macro DSL to define layouts.
- Define a fixed endianness in the layout, ensuring cross platform compatibility.
- Fully written in safe Rust, no [std::mem::transmute] or similar shenanigans.
- Const generics make sure that all offset calculations happen at compile time and will not have a runtime overhead.
- Comprehensive test coverage.

## Why not `#[repr(packed)]`?
Annotating structs with `#[repr(packed)]` gives some of the features of this crate, namely it lays out the data fields exactly in the order they're specified
without padding. But it has serious shortcomings that this library solves.
- `#[repr(packed)]` uses the system byte order, which will be different depending on if you're running on a little endian or big endian system. `#[repr(packed)]` is not cross-platform compatible. This library is.
- `#[repr(packed)]` [can cause undefined behavior on some CPUs when taking references to unaligned data](https://doc.rust-lang.org/nomicon/other-reprs.html#reprpacked).
   This library avoids that by not offering any API that takes references to unaligned data. The only data type you can get a reference to is byte arrays, and they only require an alignment of 1 which is trivially always fulfilled.

## When not to use this library?
- You need dynamic data structures, e.g. a list that can change size. This library only supports static data layouts.
- Not all of your data fits into the memory and you need to process streams of data.

## Alternatives
To the best of my knowledge, there is no other library offering inplace, zero-copy and type-safe access to structured binary data.
But if you don't need direct access to your data and are ok with a serialization/deserialization step, then there is a number of amazing libraries out there.
- [Nom](https://crates.io/crates/nom) is a great crate for all your parsing needs. It can for example parse binary data and put them in your custom structs.
- [Binread](https://crates.io/crates/binread), [Binwrite](https://crates.io/crates/binwrite), [Binrw](https://crates.io/crates/binrw) are great libraries for (de)serializing binary data.

## APIs
This library offers two alternative APIs:
1. The [Field](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html) API that offers free functions to read/write the data based on an underlying slice of storage (`packet_data` in the example above) holding the packet data. This API does not wrap the underlying slice of storage data, which means you have to pass it in to each accessor.
   This is not the API used in the example above, see [Field](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html) for an API example.
2. The [FieldView](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html) API that wraps a slice of storage data and remembers it in a `View` object, allowing access to the fields without having to pass in the packed data slice each time. This is the API used in the example above. See [FieldView](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html) for another example.

## Supported field types
### Primitive integer types
- [u8](https://doc.rust-lang.org/stable/std/primitive.u8.html), [u16](https://doc.rust-lang.org/stable/std/primitive.u16.html), [u32](https://doc.rust-lang.org/stable/std/primitive.u32.html), [u64](https://doc.rust-lang.org/stable/std/primitive.u64.html)
- [i8](https://doc.rust-lang.org/stable/std/primitive.i8.html), [i16](https://doc.rust-lang.org/stable/std/primitive.i16.html), [i32](https://doc.rust-lang.org/stable/std/primitive.i32.html), [i64](https://doc.rust-lang.org/stable/std/primitive.i64.html)

For these fields, the [Field](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html) API offers [Field::read](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html#method.read), [Field::write](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html#method.write) and the [FieldView](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html) API offers [FieldView::read](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html#method.read) and [FieldView::write](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html#method.write).

### Fixed size byte arrays: `[u8; N]`.
For these fields, the [Field](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html) API offers [Field::data](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html#method.data), [Field::data_mut](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html#method.data_mut), and the [FieldView](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html) API offers [FieldView::data](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html#method.data) and [FieldView::data_mut](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html#method.data_mut).

### Open ended byte arrays: `[u8]`.
This field type can only occur as the last field of a layout and will mach the remaining data until the end of the storage.
This field has a dynamic size, depending on how large the package data is.
For these fields, the [Field](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html) API offers [Field::data](https://docs.rs/binary-layout/latest/binary_layout/struct.Field.html#method.data), [Field::data_mut](Field::data_mut) and the [FieldView](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html) API offers [FieldView::data](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html#method.data), [FieldView::data_mut](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html#method.data_mut) and [FieldView::extract](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html#method.extract).

## Data types maybe supported in the future
These data types aren't supported yet, but they could be added in theory and might be added in future versions.
- [bool](https://doc.rust-lang.org/stable/std/primitive.bool.html) stored as 1 byte
- [bool](https://doc.rust-lang.org/stable/std/primitive.bool.html) stored as 1 bit

### Data types with dynamic length
This crate relies on a static layout, it cannot support data types with dynamic length.
In theory, types with dynamic length could be supported if they either
- are the last field of a layout, an already implemented example of this are open ended byte arrays.
- or they may be in the middle of the package but have a maximal size defined and will always reserve storage for their maximal size, even if smaller.
  This way, the fields after it would still have a constant offset.

Both of these, however, would be some effort to implement and it is unclear if that will ever happen (unless somebody opens a PR for it).

### Strings
For strings, note that even fixed-size UTF-8 strings take a variable number of characters because of the UTF-8 encoding and that brings all the issues of data types with dynamic length with it.
This is why strings aren't supported yet.

### Fixed-size arrays other than `[u8; N]`
Say we wanted to have a `[u32; N]` field. The API couldn't just return a zero-copy `&[u32; N]` to the caller because that would use the system byte order (i.e. endianness) which might be different from the byte order defined in the package layout.
To make this cross-platform compatible, we'd have to wrap these slices into our own slice type that enforces the correct byte order and return that from the API.
This complexity is why it wasn't implemented yet, but feel free to open a PR if you need this.

License: MIT OR Apache-2.0
