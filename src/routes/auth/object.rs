use std::env;

use axum::{
    async_trait, 
    extract::{FromRequestParts, TypedHeader, State, FromRef}, 
    headers::{Authorization, authorization::Bearer}, 
    http::request::Parts,
    RequestPartsExt,
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use jsonwebtoken::{EncodingKey, DecodingKey, decode, Validation, errors::{Error, ErrorKind}};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use sea_orm::{entity::*, query::*, DatabaseConnection};
use redis::AsyncCommands;

use crate::{AppState, redis_schema, db::macros::RedisSchemaHeader};
use crate::entity;

use super::AuthError;

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub email    : String,
    pub password : String,
    pub nickname : String,
}
#[derive(Debug, Deserialize)]
pub struct IssuePayload {
    pub email    : String,
    pub password : String,
}

#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub access_token  : String,
    pub refresh_token : String,
}
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token : String,
}
pub struct Keys {
    pub encoding : EncodingKey,
    pub decoding : DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Keys {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub static ACCESS_KEYS: Lazy<Keys> = Lazy::new(||{
    tracing::debug!("initializing access keys");
    Keys::new(env::var("ACCESS_JWT_SECRET").expect("ACCESS_JWT_SECRET is not set").as_bytes())
});
pub static REFRESH_KEYS: Lazy<Keys> = Lazy::new(||{
    tracing::debug!("initializing refresh keys");
    Keys::new(env::var("REFRESH_JWT_SECRET").expect("REFRESH_JWT_SECRET is not set").as_bytes())
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iat       : i64,
    pub exp       : i64,
    pub iss       : String,
    pub user_id   : i32,
    pub token_typ : String,

}
#[async_trait]
impl<S> FromRequestParts<S> for Claims
    where 
    DatabaseConnection: FromRef<S>,
    Pool<RedisConnectionManager>: FromRef<S>,
    S: Sync + Send
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::TokenMissing)?;

        let token_data = decode::<Claims>(bearer.token(), &ACCESS_KEYS.decoding, &Validation::default())
            .map_err(|err| {
                if err.into_kind() == ErrorKind::ExpiredSignature {
                    AuthError::TokenExpired
                }
                else {
                    AuthError::InvalidToken
                }
            })?;

        let con = Pool::<RedisConnectionManager>::from_ref(state);
        let redis_header = RedisSchemaHeader {
            scope: "blacklist".to_string(),
            key: bearer.token().to_string(),
            expire_at: None, 
            con,
        };
        let mut redis_schema = redis_schema!(redis_header, {status: bool});
        redis_schema.get_status().await;
        if redis_schema.status.is_some() {
            return Err(AuthError::InvalidToken);
        }
        

        let qr = entity::docuser::Entity::find()
            .filter(entity::docuser::Column::Id.eq(token_data.claims.user_id.clone()))
            .one(&DatabaseConnection::from_ref(state))
            .await?;
        if qr.is_none() {
            return Err(AuthError::InvalidCredential);
        }

        Ok(token_data.claims)
    }
}
