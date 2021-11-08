use std::marker::PhantomData;

use super::Endianness;

pub struct NestedField<T: ?Sized, E: Endianness, const OFFSET_: usize> {
    _p1: PhantomData<T>,
    _p2: PhantomData<E>,
}
