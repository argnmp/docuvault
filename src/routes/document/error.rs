use axum::{response::IntoResponse, http::StatusCode};

#[derive(Debug)]
pub enum DocumentError {
    ScopeNotExist,
    DocumentNotExist,
    PublishTokenMissing,
    PublishTokenExpired,
    InvalidPublishToken,
    JwtCreationError,
}
impl IntoResponse for DocumentError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::ScopeNotExist => (StatusCode::BAD_REQUEST, "specified scope does not exists."),
            Self::DocumentNotExist => (StatusCode::BAD_REQUEST, "target document not exists."),
            Self::PublishTokenMissing => (StatusCode::UNAUTHORIZED, "publish token is missing."),
            Self::PublishTokenExpired => (StatusCode::UNAUTHORIZED, "publish token is expired."),
            Self::InvalidPublishToken => (StatusCode::UNAUTHORIZED, "invalid publish token."),
            Self::JwtCreationError => (StatusCode::INTERNAL_SERVER_ERROR, "jwt creation failed.")
        };
        res.into_response()
    }
}
impl From<jsonwebtoken::errors::Error> for DocumentError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        DocumentError::JwtCreationError    
    }
}
