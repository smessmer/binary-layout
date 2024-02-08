pub trait SliceRead {
    fn data(storage: &[u8]) -> &Self;
}

pub trait SliceWrite {
    fn data_mut(storage: &mut [u8]) -> &mut Self;
}

mod fixed_size;
mod open_ended;
