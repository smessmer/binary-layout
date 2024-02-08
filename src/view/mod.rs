use crate::Field;

/// TODO Docs
pub trait FieldView<S, F: Field> {
    /// TODO Docs
    fn new(storage: S) -> Self;
}

mod primitive;
pub use primitive::PrimitiveFieldView;
