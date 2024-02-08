use crate::LayoutAs;
use core::convert::Infallible;

/// This error is thrown when trying to read a bool that isn't `0` or `1`.
#[derive(Debug)]
pub struct InvalidBoolError(pub(crate) ());

impl core::fmt::Display for InvalidBoolError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "InvalidBoolError")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidBoolError {}

impl LayoutAs<u8> for bool {
    type ReadError = InvalidBoolError;
    type WriteError = Infallible;

    fn try_read(v: u8) -> Result<Self, Self::ReadError> {
        match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(InvalidBoolError(())),
        }
    }

    fn try_write(v: Self) -> Result<u8, Self::WriteError> {
        match v {
            true => Ok(1),
            false => Ok(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{binary_layout, WrappedFieldError};

    const INVALID_BOOL: u8 = 3;

    macro_rules! test_bool {
        ($endian:ident, $endian_type:ty, $from_endian_fn:ident, $to_endian_fn:ident) => {
            paste::paste! {
                #[allow(non_snake_case)]
                #[test]
                fn [<test_bool_ $endian endian_viewapi_tryread_write>]() {
                    binary_layout!(layout, $endian_type, {
                        field1: bool as u8,
                        field2: bool as u8,
                        field3: bool as u8,
                    });
                    let mut storage = [0; 1024];
                    storage[2..3].copy_from_slice(&INVALID_BOOL.$to_endian_fn()); // Invalid unicode code point into field3

                    let mut view = layout::View::new(&mut storage);

                    view.field1_mut().write(true);
                    view.field2_mut().write(false);

                    assert_eq!(true, view.field1().try_read().unwrap());
                    assert_eq!(false, view.field2().try_read().unwrap());
                    assert!(matches!(view.field3().try_read(), Err(WrappedFieldError::LayoutAsError(InvalidBoolError(_)))));

                    assert_eq!(1, u8::$from_endian_fn((&storage[0..1]).try_into().unwrap()));
                    assert_eq!(0, u8::$from_endian_fn((&storage[1..2]).try_into().unwrap()));
                    assert_eq!(INVALID_BOOL, u8::$from_endian_fn((&storage[2..3]).try_into().unwrap()));
                }

                #[allow(non_snake_case)]
                #[test]
                fn [<test_bool_ $endian endian_viewapi_tryread_trywrite>]() {
                    binary_layout!(layout, $endian_type, {
                        field1: bool as u8,
                        field2: bool as u8,
                        field3: bool as u8,
                    });
                    let mut storage = [0; 1024];
                    storage[2..3].copy_from_slice(&INVALID_BOOL.$to_endian_fn()); // Invalid unicode code point into field3

                    let mut view = layout::View::new(&mut storage);

                    view.field1_mut().write(true);
                    view.field2_mut().write(false);

                    assert_eq!(true, view.field1().try_read().unwrap());
                    assert_eq!(false, view.field2().try_read().unwrap());
                    assert!(matches!(view.field3().try_read(), Err(WrappedFieldError::LayoutAsError(InvalidBoolError(_)))));

                    assert_eq!(1, u8::$from_endian_fn((&storage[0..1]).try_into().unwrap()));
                    assert_eq!(0, u8::$from_endian_fn((&storage[1..2]).try_into().unwrap()));
                    assert_eq!(INVALID_BOOL, u8::$from_endian_fn((&storage[2..3]).try_into().unwrap()));
                }
            }
        }
    }

    test_bool!(little, LittleEndian, from_le_bytes, to_le_bytes);
    test_bool!(big, BigEndian, from_be_bytes, to_be_bytes);
    test_bool!(native, NativeEndian, from_ne_bytes, to_ne_bytes);
}
