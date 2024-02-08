use std::marker::PhantomData;
use std::ops::Deref;

use super::FieldView;
use crate::{Field, FieldCopyAccess, FieldReadExt, FieldSliceAccess, FieldWriteExt};

pub struct PrimitiveFieldView<S, F: Field> {
    storage: S,
    _p: PhantomData<F>,
}

impl<S, F: Field> FieldView<S, F> for PrimitiveFieldView<S, F> {
    fn new(storage: S) -> Self {
        Self {
            storage,
            _p: PhantomData,
        }
    }
}

impl<S: AsRef<[u8]>, F: Field + FieldCopyAccess> PrimitiveFieldView<S, F> {
    #[inline(always)]
    pub fn try_read(&self) -> Result<F::HighLevelType, F::ReadError> {
        F::try_read(self.storage.as_ref())
    }
}

impl<S: AsRef<[u8]>, F: Field + FieldCopyAccess + FieldReadExt> PrimitiveFieldView<S, F> {
    #[inline(always)]
    pub fn read(&self) -> <F as FieldReadExt>::HighLevelType {
        F::read(self.storage.as_ref())
    }
}

impl<S: AsMut<[u8]>, F: Field + FieldCopyAccess> PrimitiveFieldView<S, F> {
    #[inline(always)]
    pub fn try_write(&mut self, value: F::HighLevelType) -> Result<(), F::WriteError> {
        F::try_write(self.storage.as_mut(), value)
    }
}

impl<S: AsMut<[u8]>, F: Field + FieldCopyAccess + FieldWriteExt> PrimitiveFieldView<S, F> {
    #[inline(always)]
    pub fn write(&mut self, value: <F as FieldWriteExt>::HighLevelType) {
        F::write(self.storage.as_mut(), value);
    }
}
