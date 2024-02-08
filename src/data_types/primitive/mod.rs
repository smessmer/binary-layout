use crate::endianness::Endianness;

pub trait PrimitiveRead: Sized {
    type Error;

    fn try_read<E: Endianness>(storage: &[u8]) -> Result<Self, Self::Error>;
}

pub trait PrimitiveWrite: Sized {
    type Error;

    fn try_write<E: Endianness>(self, storage: &mut [u8]) -> Result<(), Self::Error>;
}

mod float;
mod int;
mod nonzero_int;
mod unit;

pub use nonzero_int::NonZeroIsZeroError;
