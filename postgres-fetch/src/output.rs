use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardinalityError {
    pub expected: &'static str,
    pub actual: usize,
}

impl fmt::Display for CardinalityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "expected {}, received {} rows",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for CardinalityError {}

#[doc(hidden)]
pub trait FromRows<V>: Sized {
    fn from_rows(rows: Vec<V>) -> Result<Self, CardinalityError>;
}

impl<V> FromRows<V> for V {
    fn from_rows(mut rows: Vec<V>) -> Result<Self, CardinalityError> {
        if rows.len() != 1 {
            return Err(CardinalityError {
                expected: "exactly one row",
                actual: rows.len(),
            });
        }
        Ok(rows.pop().expect("one row should be present"))
    }
}

impl<V> FromRows<V> for Option<V> {
    fn from_rows(mut rows: Vec<V>) -> Result<Self, CardinalityError> {
        if rows.len() > 1 {
            return Err(CardinalityError {
                expected: "zero or one row",
                actual: rows.len(),
            });
        }
        Ok(rows.pop())
    }
}

impl<V> FromRows<V> for Vec<V> {
    fn from_rows(rows: Vec<V>) -> Result<Self, CardinalityError> {
        Ok(rows)
    }
}
