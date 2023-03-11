use std::error::Error;
use std::fmt::Display;

use axum::{
    response::IntoResponse,
    http::StatusCode,
};
use redis::RedisError;

use super::auth::error::AuthError;
use super::document::error::DocumentError;
use super::file::error::FileError;
use super::resource::error::ResourceError;

#[derive(Debug)]
pub enum GlobalError {
    InternalServerError,
    NoPermission,
    DbError,
    DbTrxError,
    RedisError,
    RedisConnectionPoolError,
    GrpcError(String),
    Auth(AuthError),
    Document(DocumentError),
    Resource(ResourceError),
    File(FileError),
}
impl Display for GlobalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for GlobalError {}
impl IntoResponse for GlobalError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response(),
            Self::NoPermission => (StatusCode::FORBIDDEN, "no permission").into_response(),
            Self::DbError => (StatusCode::INTERNAL_SERVER_ERROR, "Db error").into_response(),
            Self::DbTrxError => (StatusCode::INTERNAL_SERVER_ERROR, "Db transaction error").into_response(),
            Self::RedisError => (StatusCode::INTERNAL_SERVER_ERROR, "Redis error").into_response(),
            Self::RedisConnectionPoolError => (StatusCode::INTERNAL_SERVER_ERROR, "Redis bb8 connection pool error").into_response(),
            Self::GrpcError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
            Self::Auth(error) => error.into_response(),
            Self::Document(error) => error.into_response(),
            Self::Resource(error) => error.into_response(),
            Self::File(error) => error.into_response(),
        }
    }
}

impl From<sea_orm::error::DbErr> for GlobalError {
    fn from(value: sea_orm::error::DbErr) -> Self {
        dbg!(value);
        GlobalError::DbError
    }
}
impl<E> From<sea_orm::TransactionError<E>> for GlobalError where E: Error{
    fn from(value: sea_orm::TransactionError<E>) -> Self {
        // need to be modified for returning resource or document or auth errors
        dbg!(value); 
        GlobalError::DbTrxError
    }
}
impl From<RedisError> for GlobalError {
    fn from(value: RedisError) -> Self {
        dbg!(value);
        Self::RedisError
    }
}
impl<E> From<bb8::RunError<E>> for GlobalError {
    fn from(value: bb8::RunError<E>) -> Self {
        Self::RedisConnectionPoolError 
    }
}
impl From<tonic::Status> for GlobalError {
    fn from(value: tonic::Status) -> Self {
        dbg!(&value);
        Self::GrpcError(value.message().to_owned())
    }
}
impl From<tonic::transport::Error> for GlobalError {
    fn from(value: tonic::transport::Error) -> Self {
        dbg!(value);
        Self::InternalServerError
    }
}
