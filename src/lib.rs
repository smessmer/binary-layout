//! The [binary-layout](https://crates.io/crates/binary-layout) library allows type-safe, inplace, zero-copy access to structured binary data.
//! You define a custom data layout and give it a slice of binary data, and it will allow you to read and
//! write the fields defined in the layout from the binary data without having to copy any of the data.
//! It's similar to transmuting to/from a `#[repr(packed)]` struct, but [much safer](#why-not-reprpacked).
//!
//! Note that the data does not go through serialization/deserialization or a parsing step.
//! All accessors access the underlying packet data directly.
//!
//! This crate is `#[no_std]` compatible.
//!
//! # Example
//! ```
//! use binary_layout::prelude::*;
//!
//! // See https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol for ICMP packet layout
//! binary_layout!(icmp_packet, BigEndian, {
//!   packet_type: u8,
//!   code: u8,
//!   checksum: u16,
//!   rest_of_header: [u8; 4],
//!   data_section: [u8], // open ended byte array, matches until the end of the packet
//! });
//!
//! fn func(packet_data: &mut [u8]) {
//!   let mut view = icmp_packet::View::new(packet_data);
//!
//!   // read some data
//!   let code: u8 = view.code().read();
//!   // equivalent: let code: u8 = packet_data[1];
//!
//!   // write some data
//!   view.checksum_mut().write(10);
//!   // equivalent: packet_data[2..4].copy_from_slice(&10u16.to_be_bytes());
//!
//!   // access an open ended byte array
//!   let data_section: &[u8] = view.data_section();
//!   // equivalent: let data_section: &[u8] = &packet_data[8..];
//!
//!   // and modify it
//!   view.data_section_mut()[..5].copy_from_slice(&[1, 2, 3, 4, 5]);
//!   // equivalent: packet_data[8..13].copy_from_slice(&[1, 2, 3, 4, 5]);
//! }
//! ```
//!
//! See the [icmp_packet](crate::example::icmp_packet) module for what this [binary_layout!] macro generates for you.
//!
//! # What to use this library for?
//! Anything that needs inplace zero-copy access to structured binary data.
//! - Network packets are an obvious example
//! - File system inodes
//! - Structured binary data in files if you want to avoid explicit (de)serialization, possibly in combination with [memmap](https://docs.rs/memmap).
//!
//! ## Why use this library?
//! - Inplace, zero-copy, type-safe access to your data.
//! - Data layout is defined in one central place, call sites can't accidentally use wrong field offsets.
//! - Convenient and simple macro DSL to define layouts.
//! - Define a fixed endianness in the layout, ensuring cross platform compatibility.
//! - Fully written in safe Rust, no [std::mem::transmute](https://doc.rust-lang.org/std/mem/fn.transmute.html) or similar shenanigans.
//! - Const generics make sure that all offset calculations happen at compile time and will not have a runtime overhead.
//! - Comprehensive test coverage.
//!
//! ## Why not `#[repr(packed)]`?
//! Annotating structs with `#[repr(packed)]` gives some of the features of this crate, namely it lays out the data fields exactly in the order they're specified
//! without padding. But it has serious shortcomings that this library solves.
//! - `#[repr(packed)]` uses the system byte order, which will be different depending on if you're running on a little endian or big endian system. `#[repr(packed)]` is not cross-platform compatible. This library is.
//! - `#[repr(packed)]` [can cause undefined behavior on some CPUs when taking references to unaligned data](https://doc.rust-lang.org/nomicon/other-reprs.html#reprpacked).
//!    This library avoids that by not offering any API that takes references to unaligned data. Primitive integer types are allowed to be unaligned but they're copied and you can't get references to them.
//!    The only data type you can get a reference to is byte arrays, and they only require an alignment of 1 which is trivially always fulfilled.
//!
//! ## When not to use this library?
//! - You need dynamic data structures, e.g. a list that can change size. This library only supports static data layouts (with the exception of open ended byte arrays at the end of a layout).
//! - Not all of your layout fits into the memory and you need to process streams of data.
//!   Note that this crate can still be helpful if you have smaller layouted packets as part of a larger stream, as long as any one layouted packet fits into memory.
//!
//! ## Alternatives
//! To the best of my knowledge, there is no other library offering inplace, zero-copy and type-safe access to structured binary data.
//! But if you don't need direct access to your data and are ok with a serialization/deserialization step, then there is a number of amazing libraries out there.
//! - [Nom](https://crates.io/crates/nom) is a great crate for all your parsing needs. It can for example parse binary data and put them in your custom structs.
//! - [Binread](https://crates.io/crates/binread), [Binwrite](https://crates.io/crates/binwrite), [Binrw](https://crates.io/crates/binrw) are great libraries for (de)serializing binary data.
//!
//! # APIs
//! Layouts are defined using the [binary_layout!] macro. Based on such a layout, this library offers two alternative APIs for data access:
//! 1. The [trait@Field] API that offers free functions to read/write the data based on an underlying slice of storage (`packet_data` in the example above) holding the packet data. This API does not wrap the underlying slice of storage data, which means you have to pass it in to each accessor.
//!    This is not the API used in the example above, see [trait@Field] for an API example.
//! 2. The [struct@FieldView] API that wraps a slice of storage data and remembers it in a `View` object, allowing access to the fields without having to pass in the packed data slice each time. This is the API used in the example above. See [struct@FieldView] for another example.
//!
//! ## Supported field types
//!
//! ### Primitive integer types
//! - [u8](https://doc.rust-lang.org/stable/core/primitive.u8.html), [u16](https://doc.rust-lang.org/stable/core/primitive.u16.html), [u32](https://doc.rust-lang.org/stable/core/primitive.u32.html), [u64](https://doc.rust-lang.org/stable/core/primitive.u64.html), [u128](https://doc.rust-lang.org/stable/core/primitive.u128.html)
//! - [i8](https://doc.rust-lang.org/stable/core/primitive.i8.html), [i16](https://doc.rust-lang.org/stable/core/primitive.i16.html), [i32](https://doc.rust-lang.org/stable/core/primitive.i32.html), [i64](https://doc.rust-lang.org/stable/core/primitive.i64.html), [i128](https://doc.rust-lang.org/stable/core/primitive.i128.html)
//!
//! For these fields, the [trait@Field] API offers [FieldReadExt::read], [FieldWriteExt::write], [FieldCopyAccess::try_read], [FieldCopyAccess::try_write] and the [struct@FieldView] API offers [FieldView::read] and [FieldView::write].
//!
//! ### Primitive float types
//! - [f32](https://doc.rust-lang.org/core/primitive.f32.html), [f64](https://doc.rust-lang.org/core/primitive.f64.html)
//!
//! ### Non-zero primitive integer types
//! - [NonZeroU8](https://doc.rust-lang.org/core/num/struct.NonZeroU8.html), [NonZeroU16](https://doc.rust-lang.org/core/num/struct.NonZeroU16.html), [NonZeroU32](https://doc.rust-lang.org/core/num/struct.NonZeroU32.html), [NonZeroU64](https://doc.rust-lang.org/core/num/struct.NonZeroU64.html), [NonZeroU128](https://doc.rust-lang.org/core/num/struct.NonZeroU128.html)
//! - [NonZeroI8](https://doc.rust-lang.org/core/num/struct.NonZeroI8.html), [NonZeroI16](https://doc.rust-lang.org/core/num/struct.NonZeroI16.html), [NonZeroI32](https://doc.rust-lang.org/core/num/struct.NonZeroI32.html), [NonZeroI64](https://doc.rust-lang.org/core/num/struct.NonZeroI64.html), [NonZeroI128](https://doc.rust-lang.org/core/num/struct.NonZeroI128.html)
//!
//! Reading a zero values will throw an error. Because of this, [FieldReadExt::read] and [FieldView::read] are not available for those types and you need to use [FieldCopyAccess::try_read] and [FieldView::try_read].
//!
//! ### bool, char
//! [bool](https://doc.rust-lang.org/stable/core/primitive.bool.html) and [char](https://doc.rust-lang.org/stable/core/primitive.char.html) are supported using the `bool as u8` and `char as u32` data type notation.
//!
//! Note that not only `0u8` and `1u8` are valid boolean values and not all [u32](https://doc.rust-lang.org/stable/core/primitive.u32.html) values are valid unicode code points.
//! Reading invalid values will throw an error. Because of this, [FieldReadExt::read] and [FieldView::read] are not available for those types and you need to use [FieldCopyAccess::try_read] and [FieldView::try_read].
//!
//! ### Primitive Zero-Sized Types (ZSTs)
//!
//! ZSTs neither read nor write to the underlying storage, but the appropriate traits are implemented for them to support derive macros which may require all members of a struct to implement or enum to also support the various traits.
//!
//! - [`()`](https://doc.rust-lang.org/core/primitive.unit.html), also known as the `unit` type.
//!
//! ### Fixed size byte arrays: `[u8; N]`.
//! For these fields, the [trait@Field] API offers [FieldSliceAccess::data], [FieldSliceAccess::data_mut], and the [struct@FieldView] API returns a slice.
//!
//! ### Open ended byte arrays: `[u8]`.
//! This field type can only occur as the last field of a layout and will mach the remaining data until the end of the storage.
//! This field has a dynamic size, depending on how large the packet data is.
//! For these fields, the [trait@Field] API offers [FieldSliceAccess::data], [FieldSliceAccess::data_mut] and the [struct@FieldView] API returns a slice.
//!
//! ### Custom field types
//! You can define your own custom types as long as they implement the [trait@LayoutAs] trait to define how to convert them from/to a primitive type.
//!
//! # Data types maybe supported in the future
//! These data types aren't supported yet, but they could be added in theory and might be added in future versions.
//! - bit fields / [bool](https://doc.rust-lang.org/stable/core/primitive.bool.html) stored as 1 bit
//!
//! ### Data types with dynamic length
//! This crate relies on a static layout, it cannot support data types with dynamic length.
//! In theory, types with dynamic length could be supported if they either
//! - are the last field of a layout, an already implemented example of this are open ended byte arrays.
//! - or they may be in the middle of the packet but have a maximal size defined and will always reserve storage for their maximal size, even if smaller.
//!   This way, the fields after it would still have a constant offset.
//!
//! Both of these, however, would be some effort to implement and it is unclear if that will ever happen (unless somebody opens a PR for it).
//!
//! ### Strings
//! For strings, note that even fixed-size UTF-8 strings take a variable number of bytes because of the UTF-8 encoding and that brings all the issues of data types with dynamic length with it.
//! This is why strings aren't supported yet.
//!
//! ### Fixed-size arrays other than `[u8; N]`
//! Say we wanted to have a `[u32; N]` field. The API couldn't just return a zero-copy `&[u32; N]` to the caller because that would use the system byte order (i.e. endianness) which might be different from the byte order defined in the packet layout.
//! To make this cross-platform compatible, we'd have to wrap these slices into our own slice type that enforces the correct byte order and return that from the API.
//! This complexity is why it wasn't implemented yet, but feel free to open a PR if you need this.
//!
//! # Nesting
//! Layouts can be nested within each other by using the `NestedView` type created by the [binary_layout!] macro for one layout as a field type in another layout.
//!
//! Example:
//! ```
//! use binary_layout::prelude::*;
//!
//! binary_layout!(icmp_header, BigEndian, {
//!   packet_type: u8,
//!   code: u8,
//!   checksum: u16,
//!   rest_of_header: [u8; 4],
//! });
//! binary_layout!(icmp_packet, BigEndian, {
//!   header: icmp_header::NestedView,
//!   data_section: [u8], // open ended byte array, matches until the end of the packet
//! });
//! # fn main() {}
//! ```
//!
//! Nested layouts do not need to have the same endianess.  The following, which
//! is copied from the complete example at `tests/nested.rs` in this repository,
//! shows how you can mix different endian layouts together:
//!
//! ```rust
//! use binary_layout::prelude::*;
//! use core::convert::TryInto;
//!
//! binary_layout!(deep_nesting, LittleEndian, {
//!     field1: u16,
//! });
//! binary_layout!(header, BigEndian, {
//!     field1: i16,
//! });
//! binary_layout!(middle, NativeEndian, {
//!     deep: deep_nesting::NestedView,
//!     field1: u16,
//! });
//! binary_layout!(footer, BigEndian, {
//!     field1: u32,
//!     deep: deep_nesting::NestedView,
//!     tail: [u8],
//! });
//! binary_layout!(whole, LittleEndian, {
//!     head: header::NestedView,
//!     field1: u64,
//!     mid: middle::NestedView,
//!     field2: u128,
//!     foot: footer::NestedView,
//! });
//! # fn main() {}
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod data_types;
mod endianness;
mod fields;
mod macro_binary_layout;
mod utils;
mod view;

pub mod example;

pub use data_types::primitive::NonZeroIsZeroError;
pub use endianness::{BigEndian, Endianness, LittleEndian, NativeEndian};
pub use fields::{
    primitive::{FieldCopyAccess, FieldReadExt, FieldSliceAccess, FieldWriteExt, PrimitiveField},
    //wrapped::{LayoutAs, WrappedField, WrappedFieldError},
    Field,
};
pub use utils::{data::Data, infallible::InfallibleResultExt};
pub use view::FieldView;

/// Import this to get everything into scope that you need for defining and using layouts.
///
/// # Example
/// ```
/// use binary_layout::prelude::*;
/// ```
pub mod prelude {
    pub use super::{
        BigEndian, Field, FieldCopyAccess, FieldReadExt, FieldSliceAccess, FieldWriteExt,
        InfallibleResultExt, LittleEndian, NativeEndian, NonZeroIsZeroError,
    };
    pub use crate::binary_layout;
    #[allow(deprecated)]
    pub use crate::define_layout;
}

/// Internal things that need to be exported so our macros can use them. Don't use directly!
#[doc(hidden)]
pub mod internal {
    pub use crate::macro_binary_layout::{option_usize_add, unwrap_field_size};
    pub use doc_comment::doc_comment;
    pub use paste::paste;
}
