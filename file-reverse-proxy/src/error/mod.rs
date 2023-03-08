use std::{error::Error, fmt::Display};
use tonic::{Status, Code};

#[derive(Debug)]
pub enum GlobalError {
    IoError,
}

impl Display for GlobalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for GlobalError {}
impl From<std::io::Error> for GlobalError {
    fn from(value: std::io::Error) -> Self {
        dbg!(value);
        Self::IoError
    }
}
impl From<GlobalError> for Status {
    fn from(value: GlobalError) -> Self {
        match value {
            GlobalError::IoError => Status::new(Code::Internal, "io error"),
        }
    }
}


