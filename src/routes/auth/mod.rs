use std::env;

use axum::{Router, routing::{get, post}, response::{Html, IntoResponse}, extract::State, Json};
use jsonwebtoken::{EncodingKey, DecodingKey, encode, Header};
use sea_orm::{entity::*, query::*};

use regex::Regex;
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::json;


use crate::AppState;
use crate::entity;

mod error;
use error::*;
mod object;
use object::*;

use self::module::password::verify_password;
mod module;


pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/protected", get(protected))
        .route("/register", post(register))
        .route("/issue", post(issue))
        .with_state(shared_state)
}

async fn protected(claims: Claims) -> impl IntoResponse {
    dbg!(claims);
}
async fn index(State(state): State<AppState>) -> impl IntoResponse {
    Html("welcome to auth index")
}

async fn register(State(state): State<AppState>, Json(payload): Json<RegisterPayload>) -> Result<impl IntoResponse, error:: AuthError> {
    if payload.email.is_empty() || payload.password.is_empty() || payload.nickname.is_empty() {
        return Err(AuthError::MissingCredential);
    }
    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(AuthError::InvalidCredential);  
    }

    let qr = entity::docuser::Entity::find()
        .filter(entity::docuser::Column::Email.eq(payload.email.clone()))
        .all(&state.db_conn)
        .await?;
    if qr.len() >= 1 {
        return Err(AuthError::DuplicateEmail);
    }
    
    let password_hash = module::password::create_hash(&payload.password[..].as_bytes())?;
    let new_user = entity::docuser::ActiveModel {
        email: Set(payload.email), 
        nickname: Set(payload.nickname),
        hash: Set(password_hash),
        ..Default::default()
    };
    let insert_result = entity::docuser::Entity::insert(new_user).exec(&state.db_conn).await?;
    Ok(())

}
async fn issue(State(state): State<AppState>, Json(payload): Json<IssuePayload>) -> Result<impl IntoResponse, error::AuthError> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredential);
    }
    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(AuthError::InvalidCredential);  
    }

    let qr = entity::docuser::Entity::find()
        .filter(entity::docuser::Column::Email.eq(payload.email.clone()))
        .one(&state.db_conn)
        .await?;
    if qr.is_none() {
        return Err(AuthError::InvalidCredential);
    }
    
    verify_password(&qr.unwrap().hash, &payload.password[..].as_bytes())?;

    let claims = Claims {
        sub: payload.email.clone(),
        iat: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(10)).timestamp(),
    };     
    
    let token = encode(&Header::default(), &claims, &KEYS.encoding)?;
    
    Ok(Json(IssueResponse{
        access_token: token,
    }))
}
