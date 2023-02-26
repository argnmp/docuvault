use axum::{response::IntoResponse, http::StatusCode};

use crate::routes::error::GlobalError;

#[derive(Debug)]
pub enum ResourceError {
    UnitSizeZero,
    SequenceNotExist,
    SequenceNotSync,
}

impl IntoResponse for ResourceError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::UnitSizeZero => (StatusCode::BAD_REQUEST, "unit size must not be zero"), 
            Self::SequenceNotExist => (StatusCode::BAD_REQUEST, "specified sequence id does not exist"), 
            Self::SequenceNotSync => (StatusCode::BAD_REQUEST, "update sequence not synchronized"), 
        };
        res.into_response()
    }
}
impl From<ResourceError> for GlobalError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
