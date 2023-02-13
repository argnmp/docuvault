use axum::{response::IntoResponse, http::StatusCode};

use crate::routes::error::GlobalError;

#[derive(Debug)]
pub enum ResourceError {
    UnitSizeZero,
}

impl IntoResponse for ResourceError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::UnitSizeZero => (StatusCode::BAD_REQUEST, "unit size must not be zero"), 
        };
        res.into_response()
    }
}
impl From<ResourceError> for GlobalError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
