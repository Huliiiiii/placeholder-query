use crate::expr::ExprNode;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    TinyInt(i8),
    SmallInt(i16),
    Int(i32),
    BigInt(i64),
    TinyUnsigned(u8),
    SmallUnsigned(u16),
    Unsigned(u32),
    BigUnsigned(u64),
    Float(f32),
    Double(f64),
    Text(String),
    Bytes(Vec<u8>),
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Self::TinyInt(value)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Self::SmallInt(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::BigInt(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::TinyUnsigned(value)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Self::SmallUnsigned(value)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Unsigned(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::BigUnsigned(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

impl From<Value> for ExprNode {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

macro_rules! impl_from_value_type_for_expr {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for ExprNode {
                fn from(value: $ty) -> Self {
                    Self::Value(value.into())
                }
            }
        )*
    };
}

impl_from_value_type_for_expr!(
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    f32,
    f64,
    &str,
    String,
    Vec<u8>,
);
