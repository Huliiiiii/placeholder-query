#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SqlType {
    Bool,
    Int2,
    Int4,
    Int8,
    Float4,
    Float8,
    Text,
    Bytea,
    Array(Box<Self>),
}

impl std::fmt::Display for SqlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => f.write_str("boolean"),
            Self::Int2 => f.write_str("smallint"),
            Self::Int4 => f.write_str("integer"),
            Self::Int8 => f.write_str("bigint"),
            Self::Float4 => f.write_str("real"),
            Self::Float8 => f.write_str("double precision"),
            Self::Text => f.write_str("text"),
            Self::Bytea => f.write_str("bytea"),
            Self::Array(element) => write!(f, "{element}[]"),
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

#[doc(hidden)]
pub trait ParamType: sealed::Sealed {
    fn ty() -> SqlType;
}

#[doc(hidden)]
pub trait ArrayElement: ParamType {}

macro_rules! sql_params {
    (
        $(
            $ty:ty =>
                $sql:expr,
                $array_input:ident;
        )+
    ) => {
        $(
            impl sealed::Sealed for $ty {}

            impl ParamType for $ty {
                fn ty() -> SqlType {
                    $sql
                }
            }

            impl ArrayElement for $ty {}

            impl sealed::Sealed for Option<$ty> {}

            impl ParamType for Option<$ty> {
                fn ty() -> SqlType {
                    $sql
                }
            }

            impl ArrayElement for Option<$ty> {}

            sql_params!(@array_input $array_input, $ty, $sql);
        )+
    };

    (@array_input array, $ty:ty, $sql:expr) => {
        impl sealed::Sealed for Vec<$ty> {}

        impl ParamType for Vec<$ty> {
            fn ty() -> SqlType {
                SqlType::Array(Box::new($sql))
            }
        }
    };

    (@array_input no_array, $ty:ty, $sql:expr) => {};
}

sql_params! {
    bool => SqlType::Bool, array;
    i16 => SqlType::Int2, array;
    i32 => SqlType::Int4, array;
    i64 => SqlType::Int8, array;
    f32 => SqlType::Float4, array;
    f64 => SqlType::Float8, array;
    String => SqlType::Text, array;
    Vec<u8> => SqlType::Bytea, no_array;
}
