use super::{SliceRead, SliceWrite};
use crate::data_types::DataTypeMetadata;

impl DataTypeMetadata for [u8] {
    const SIZE: Option<usize> = None;

    type View<S, F> = &[u8];
}

impl SliceRead for [u8] {
    fn data(storage: &[u8]) -> &Self {
        storage
    }
}

impl SliceWrite for [u8] {
    fn data_mut(storage: &mut [u8]) -> &mut Self {
        storage
    }
}
