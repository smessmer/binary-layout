use core::convert::Infallible;

// TODO Docs

pub(crate) trait IsInfallible {}

impl IsInfallible for Infallible {}

pub trait InfallibleResultExt<T> {
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
