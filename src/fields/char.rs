use crate::LayoutAs;
use core::convert::Infallible;

/// This error is thrown when trying to read a char that isn't a valid unicode codepoint, see [char].
#[derive(Debug)]
pub struct InvalidCharError(pub(crate) ());

impl core::fmt::Display for InvalidCharError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "InvalidCharError")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidCharError {}

impl LayoutAs<u32> for char {
    type ReadError = InvalidCharError;
    type WriteError = Infallible;

    fn try_read(v: u32) -> Result<Self, Self::ReadError> {
        char::from_u32(v).ok_or(InvalidCharError(()))
    }

    fn try_write(v: Self) -> Result<u32, Self::WriteError> {
        Ok(u32::from(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{define_layout, InfallibleResultExt, WrappedFieldError};

    const INVALID_UNICODE: u32 = 0xD83Du32;

    macro_rules! test_char {
        ($endian:ident, $endian_type:ty, $from_endian_fn:ident, $to_endian_fn:ident) => {
            paste::paste! {
                #[allow(non_snake_case)]
                #[test]
                fn [<test_char_ $endian endian_viewapi_tryread_write>]() {
                    define_layout!(layout, $endian_type, {
                        field1: char as u32,
                        field2: char as u32,
                        field3: char as u32,
                        field4: char as u32,
                    });
                    let mut storage = [0; 1024];
                    storage[12..16].copy_from_slice(&INVALID_UNICODE.$to_endian_fn()); // Invalid unicode code point into field4

                    let mut view = layout::View::new(&mut storage);

                    view.field1_mut().write('a');
                    view.field2_mut().write('我');
                    view.field3_mut().write('\0');

                    assert_eq!('a', view.field1().try_read().unwrap());
                    assert_eq!('我', view.field2().try_read().unwrap());
                    assert_eq!('\0', view.field3().try_read().unwrap());
                    assert!(matches!(view.field4().try_read(), Err(WrappedFieldError::LayoutAsError(InvalidCharError(_)))));

                    assert_eq!('a', char::try_from(u32::$from_endian_fn((&storage[0..4]).try_into().unwrap())).unwrap());
                    assert_eq!('我', char::try_from(u32::$from_endian_fn((&storage[4..8]).try_into().unwrap())).unwrap());
                    assert_eq!('\0', char::try_from(u32::$from_endian_fn((&storage[8..12]).try_into().unwrap())).unwrap());
                    assert_eq!(INVALID_UNICODE, u32::$from_endian_fn((&storage[12..16]).try_into().unwrap()));
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_char_ $endian endian_viewapi_tryread_trywrite>]() {
                    define_layout!(layout, $endian_type, {
                        field1: char as u32,
                        field2: char as u32,
                        field3: char as u32,
                        field4: char as u32,
                    });
                    let mut storage = [0; 1024];
                    storage[12..16].copy_from_slice(&INVALID_UNICODE.$to_endian_fn()); // Invalid unicode code point into field4

                    let mut view = layout::View::new(&mut storage);

                    view.field1_mut().try_write('a').infallible_unwrap();
                    view.field2_mut().try_write('我').infallible_unwrap();
                    view.field3_mut().try_write('\0').infallible_unwrap();

                    assert_eq!('a', view.field1().try_read().unwrap());
                    assert_eq!('我', view.field2().try_read().unwrap());
                    assert_eq!('\0', view.field3().try_read().unwrap());
                    assert!(matches!(view.field4().try_read(), Err(WrappedFieldError::LayoutAsError(InvalidCharError(_)))));

                    assert_eq!('a', char::try_from(u32::$from_endian_fn((&storage[0..4]).try_into().unwrap())).unwrap());
                    assert_eq!('我', char::try_from(u32::$from_endian_fn((&storage[4..8]).try_into().unwrap())).unwrap());
                    assert_eq!('\0', char::try_from(u32::$from_endian_fn((&storage[8..12]).try_into().unwrap())).unwrap());
                    assert_eq!(INVALID_UNICODE, u32::$from_endian_fn((&storage[12..16]).try_into().unwrap()));
                }
            }
        }
    }

    test_char!(little, LittleEndian, from_le_bytes, to_le_bytes);
    test_char!(big, BigEndian, from_be_bytes, to_be_bytes);
    test_char!(native, NativeEndian, from_ne_bytes, to_ne_bytes);
}
