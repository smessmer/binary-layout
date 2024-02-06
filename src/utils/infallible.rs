use core::convert::Infallible;

pub(crate) trait IsInfallible {}

impl IsInfallible for Infallible {}

/// This extension trait adds [InfallibleResultExt::infallible_unwrap] to [Result] types
/// that use [core::convert::Infallible] as error type.
pub trait InfallibleResultExt<T> {
    /// This function does the same as [Result::unwrap], but it only exists on types where the error type
    /// of the [Result] is [core::convert::Infallible]. This way, we can guarantee that this function
    /// will always be a no-op and will never panic. This is great for when your code style says that
    /// [Result::unwrap] is a code smell because it could cause runtime panics, but you need a safe
    /// alternative for it for when you know it can't fail.
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
