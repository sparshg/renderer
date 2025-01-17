use crate::core::{HasPoints, Mobject};

pub struct Animation<T: HasPoints> {
    mob: Mobject<T>,
    target: Mobject<T>,
}
