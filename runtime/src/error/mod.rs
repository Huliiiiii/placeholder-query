use std::{error::Error, fmt};

#[derive(Debug)]
pub enum FetchError<E> {
    Executor(E),
    MissingOutput { req_type: &'static str },
}

impl<E> From<E> for FetchError<E> {
    fn from(error: E) -> Self {
        Self::Executor(error)
    }
}

impl<E: fmt::Display> fmt::Display for FetchError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Executor(error) => error.fmt(f),
            Self::MissingOutput { req_type } => {
                write!(
                    f,
                    "data source returned no output for request type {req_type}"
                )
            }
        }
    }
}

impl<E: Error> Error for FetchError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Executor(error) => error.source(),
            Self::MissingOutput { .. } => None,
        }
    }
}
