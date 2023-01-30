use std::env;

use axum::{
    async_trait, 
    extract::{FromRequestParts, TypedHeader}, 
    headers::{Authorization, authorization::Bearer}, 
    http::request::Parts,
    RequestPartsExt,
};
use jsonwebtoken::{EncodingKey, DecodingKey, decode, Validation, errors::{Error, ErrorKind}};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

use super::super::error::GlobalError;
use super::AuthError;

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
    pub nickname: String,
}
#[derive(Debug, Deserialize)]
pub struct IssuePayload {
    pub email: String, 
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub access_token: String,
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}
impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Keys {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub static KEYS: Lazy<Keys> = Lazy::new(||{
    tracing::debug!("initializing keys");
    Keys::new(env::var("JWT_SECRET").expect("JWT_SECRET is not set").as_bytes())
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::TokenMissing)?;
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|err| {
                if err.into_kind() == ErrorKind::ExpiredSignature {
                    AuthError::TokenExpired
                }
                else {
                    AuthError::InvalidToken
                }
            })?;

        Ok(token_data.claims)
    }
}
