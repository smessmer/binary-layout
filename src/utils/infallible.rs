use core::convert::Infallible;

pub(crate) trait IsInfallible {}

impl IsInfallible for Infallible {}

/// TODO Docs
pub trait InfallibleResultExt<T> {
    /// TODO Docs
    fn infallible_unwrap(self) -> T;
}

impl<T, E> InfallibleResultExt<T> for Result<T, E>
where
    E: IsInfallible,
{
    fn infallible_unwrap(self) -> T {
        match self {
            Ok(value) => value,
            Err(_) => unreachable!(),
        }
    }
}
