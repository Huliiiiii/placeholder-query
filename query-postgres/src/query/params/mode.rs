use crate::query::mode::Mode;

use super::Param;

pub struct Scalar;

impl Mode for Scalar {
    type Field<T> = Param<T>;
}

pub struct Array;

impl Mode for Array {
    type Field<T> = Param<Vec<T>>;
}
