use axum::{response::IntoResponse, http::StatusCode};

use crate::routes::error::GlobalError;

#[derive(Debug)]
pub enum DocumentError {
    ScopeNotExist,
    DocumentNotExist,
    PublishTokenMissing,
    PublishTokenExpired,
    InvalidPublishToken,
    JwtCreationError,
    PrivateDocument,
    DocumentNotConverted, 
    ConvertPending,
    ConvertFailed,
}
impl IntoResponse for DocumentError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::ScopeNotExist => (StatusCode::BAD_REQUEST, "specified scope does not exists."),
            Self::DocumentNotExist => (StatusCode::BAD_REQUEST, "target document not exists."),
            Self::PublishTokenMissing => (StatusCode::UNAUTHORIZED, "publish token is missing."),
            Self::PublishTokenExpired => (StatusCode::UNAUTHORIZED, "publish token is expired."),
            Self::InvalidPublishToken => (StatusCode::UNAUTHORIZED, "invalid publish token."),
            Self::JwtCreationError => (StatusCode::INTERNAL_SERVER_ERROR, "jwt creation failed."),
            Self::PrivateDocument => (StatusCode::BAD_REQUEST, "private document"),
            Self::DocumentNotConverted => (StatusCode::NO_CONTENT, "target type is not converted"),
            Self::ConvertPending => (StatusCode::BAD_REQUEST, "target content type conversion is in process"),
            Self::ConvertFailed => (StatusCode::BAD_REQUEST, "target content type conversion failed"),
        };
        res.into_response()
    }
}
impl From<DocumentError> for GlobalError {
    fn from(value: DocumentError) -> Self {
        Self::Document(value)
    }
}
impl From<jsonwebtoken::errors::Error> for DocumentError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        DocumentError::JwtCreationError    
    }
}
