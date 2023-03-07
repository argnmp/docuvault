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
use jsonwebtoken::{EncodingKey, DecodingKey, decode, Validation, errors::ErrorKind};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sea_orm::{entity::*, query::*, FromQueryResult, DatabaseConnection};

use super::error::DocumentError;

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePayload {
    pub raw: String,
    pub tags: Vec<String>,
    pub scope_ids: Vec<i32>,
    pub seq_id: Option<i32>,
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

pub static PUBLISH_KEYS: Lazy<Keys> = Lazy::new(||{
    tracing::debug!("initializing publish keys");
    Keys::new(env::var("PUBLISH_JWT_SECRET").expect("PUBLISH_JWT_SECRET is not set").as_bytes())
});

#[derive(Debug, Deserialize)]
pub struct PublishPayload {
    pub doc_id: i32,
    pub scope_ids: Vec<i32>,
    pub c_type: i32,
}
#[derive(Debug, Serialize)]
pub struct PublishResponse {
    pub publish_token  : String,
}

#[derive(Debug,FromQueryResult)]
pub struct DocorgWithScope {
    pub id: i32,
    pub raw: String,
    pub docuser_id: i32,
    pub status: i32,
    pub scope_id: i32,
}
#[derive(Debug, Deserialize)]
pub struct GetDocumentPayload{
    pub publish_token: String,
}

pub fn get_claims(payload: GetDocumentPayload) -> Result<DocumentClaims, DocumentError>{
    let token_data = decode::<DocumentClaims>(&payload.publish_token, &PUBLISH_KEYS.decoding, &Validation::default())
        .map_err(|err| {
            if err.into_kind() == ErrorKind::ExpiredSignature {
                DocumentError::PublishTokenExpired
            }
            else {
                DocumentError::InvalidPublishToken
            }
        })?;

    Ok(token_data.claims)
    
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentClaims {
    pub iat       : i64,
    pub exp       : i64,
    pub iss       : String,
    pub token_typ : String,
    pub c_type: i32,
    pub doc_id: i32,
    pub scope_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetUpdateResourcePayload {
    pub scope_ids: Vec<i32>
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePayload {
    pub doc_id: i32,
    pub raw: String,
    pub tags: Vec<String>,
    pub scope_ids: Vec<i32>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub seq_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DeletePayload {
    pub doc_ids: Vec<i32>,
}

#[derive(Debug, FromQueryResult)]
pub struct Obj{
    pub object_id: String,
}
