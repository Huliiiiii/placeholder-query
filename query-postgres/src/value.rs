#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    SmallInt(i16),
    Int(i32),
    BigInt(i64),
    Real(f32),
    Double(f64),
    Text(String),
    Bytea(Vec<u8>),
}

macro_rules! impl_from_type_for_value {
    ($($ty:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$ty> for Value {
                fn from(value: $ty) -> Self {
                    Self::$variant(value.into())
                }
            }
        )*
    };
}

impl_from_type_for_value!(
    i8 => SmallInt,
    i16 => SmallInt,
    i32 => Int,
    i64 => BigInt,
    u8 => SmallInt,
    u16 => Int,
    u32 => BigInt,
    f32 => Real,
    f64 => Double,
    &str => Text,
    String => Text,
    Vec<u8> => Bytea,
);
