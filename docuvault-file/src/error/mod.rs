use std::{error::Error, fmt::Display};

use redis::RedisError;
use tonic::{Status, Code};

#[derive(Debug)]
pub enum GlobalError {
    DbError,
    DbTrxError,
    RedisError,
    RedisConnectionPoolError,
    ObjectNotExist,
    IoError,
}

impl Display for GlobalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for GlobalError {}
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
impl From<std::io::Error> for GlobalError {
    fn from(value: std::io::Error) -> Self {
        dbg!(value);
        Self::IoError
    }
}
impl From<GlobalError> for Status {
    fn from(value: GlobalError) -> Self {
        match value {
            GlobalError::RedisError => Status::new(Code::Internal, "redis error"),
            GlobalError::RedisConnectionPoolError => Status::new(Code::Internal, "redis connection pool error"),
            GlobalError::DbError => Status::new(Code::Internal, "database error"),
            GlobalError::DbTrxError => Status::new(Code::Internal, "database transaction error"),
            GlobalError::ObjectNotExist => Status::new(Code::NotFound, "requested object does not exist"),
            GlobalError::IoError => Status::new(Code::Internal, "io error"),
        }
    }
}

