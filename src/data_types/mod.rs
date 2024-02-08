use crate::view::FieldView;
use crate::Field;

pub trait DataTypeMetadata {
    const SIZE: Option<usize>;

    type View<S, F>: FieldView<S, F>
    where
        F: Field;
}

pub mod primitive;
pub mod slice;
