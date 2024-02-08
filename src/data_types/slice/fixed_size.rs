use super::{SliceRead, SliceWrite};
use crate::data_types::DataTypeMetadata;

impl<const N: usize> DataTypeMetadata for [u8; N] {
    const SIZE: Option<usize> = Some(N);

    type View<S, F> = &[u8; N];
}

impl<const N: usize> SliceRead for [u8; N] {
    fn data(storage: &[u8]) -> &Self {
        storage.try_into().unwrap()
    }
}

impl<const N: usize> SliceWrite for [u8; N] {
    fn data_mut(storage: &mut [u8]) -> &mut Self {
        storage.try_into().unwrap()
    }
}
