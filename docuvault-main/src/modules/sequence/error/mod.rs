use axum::{response::IntoResponse, http::StatusCode};

use crate::routes::error::GlobalError;

#[derive(Debug)]
pub enum SequenceError {
    SequenceNotExist,
}

impl IntoResponse for SequenceError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::SequenceNotExist => (StatusCode::BAD_REQUEST, "specified sequence id does not exist"),
        };
        res.into_response()
    }
}

impl From<SequenceError> for GlobalError {
    fn from(value: SequenceError) -> Self {
        Self::Sequence(value)
    }
}

