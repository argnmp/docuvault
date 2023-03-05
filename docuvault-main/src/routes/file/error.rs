use axum::{response::IntoResponse, http::StatusCode};

use crate::routes::{document::error::DocumentError, error::GlobalError};

#[derive(Debug)]
pub enum FileError {
    GrpcConnectionFail, 
    FileNotExists,
}
impl IntoResponse for FileError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::GrpcConnectionFail => (StatusCode::INTERNAL_SERVER_ERROR, "grpc connection failed"),
            Self::FileNotExists => (StatusCode::NO_CONTENT, "file not exists"),
        };
        res.into_response()
    }
}

impl From<FileError> for GlobalError {
    fn from(value: FileError) -> Self {
        Self::File(value)
    }
}


