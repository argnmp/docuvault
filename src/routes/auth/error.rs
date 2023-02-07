use axum::{
    response::IntoResponse,
    http::StatusCode,
};

#[derive(Debug)]
pub enum AuthError {
    HashError,
    JwtCreationError,
    DbError,
    RedisError,
    MissingCredential,
    InvalidCredential,
    DuplicateEmail,
    TokenMissing,
    TokenExpired,
    InvalidToken,
    IpChanged,
}
impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::MissingCredential => (StatusCode::BAD_REQUEST, "missing credential"),
            Self::InvalidCredential => (StatusCode::BAD_REQUEST, "invalid credential"),
            Self::JwtCreationError => (StatusCode::INTERNAL_SERVER_ERROR, "jwt creation error"),
            Self::HashError => (StatusCode::INTERNAL_SERVER_ERROR, "hash error"),
            Self::DbError => (StatusCode::INTERNAL_SERVER_ERROR, "Db error"),
            Self::RedisError => (StatusCode::INTERNAL_SERVER_ERROR, "Redis error"),
            Self::DuplicateEmail => (StatusCode::BAD_REQUEST, "duplicate email exists"),
            Self::TokenMissing => (StatusCode::UNAUTHORIZED, "unauthorized, token must be set."),
            Self::TokenExpired => (StatusCode::UNAUTHORIZED, "token expired"),
            Self::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid token"),
            Self::IpChanged => (StatusCode::BAD_REQUEST, "Ip changed"),
        };
        res.into_response()
    }
}
impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        AuthError::JwtCreationError    
    }
}
impl From<argon2::password_hash::Error> for AuthError {
    fn from(value: argon2::password_hash::Error) -> Self {
        AuthError::HashError    
    }
}
impl From<sea_orm::error::DbErr>for AuthError {
    fn from(value: sea_orm::error::DbErr) -> Self {
        AuthError::DbError
    }
}
impl From<redis::RedisError> for AuthError {
    fn from(value: redis::RedisError) -> Self {
        dbg!(value);
        AuthError::RedisError
    }
}
