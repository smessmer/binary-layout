use std::marker::PhantomData;

use super::IField;

/// TODO Doc
pub struct Field<U: IField> {
    _p1: PhantomData<U>,
}

impl<U: IField> IField for Field<U> {
    // TODO Doc
    type Endian = U::Endian;
    // TODO Doc
    const OFFSET: usize = U::OFFSET;
}
