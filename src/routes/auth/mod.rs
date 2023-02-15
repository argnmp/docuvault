use std::{net::SocketAddr, str::FromStr};

use axum::{Router, routing::{get, post}, response::{Html, IntoResponse}, extract::{State, ConnectInfo}, Json, middleware::{from_extractor, from_extractor_with_state}, TypedHeader, headers::{Authorization, authorization::Bearer}};
use jsonwebtoken::{encode, Header, decode, Validation, errors::ErrorKind};
use sea_orm::{entity::*, query::*};
use regex::Regex;
use serde_json::json;
use redis::{AsyncCommands};


use crate::{redis_schema, db::{schema::redis::{TokenPair, RedisSchemaHeader, Refresh, BlackList}}};
use crate::AppState;
use crate::entity;
use crate::middleware::guard::Authenticate;

pub mod error;
use error::*;
pub mod object;
use object::*;

use self::module::password::verify_password;

use super::error::GlobalError;

mod module;


pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/protected", get(protected))
        .route("/disconnect", get(disconnect))
        .route_layer(from_extractor_with_state::<Authenticate, AppState>(shared_state.clone()))
        .route("/test", get(test))
        .route("/", get(index))
        .route("/register", post(register))
        .route("/issue", post(issue))
        .route("/refresh", get(refresh))
        .with_state(shared_state)
}
async fn test(State(state): State<AppState>) -> impl IntoResponse {
}

async fn protected() -> impl IntoResponse {
    Json(json!({"msg": "you've got access"}))
}
async fn index(State(state): State<AppState>) -> impl IntoResponse {
    Html("welcome to auth index")
}

async fn register(State(state): State<AppState>, Json(payload): Json<RegisterPayload>) -> Result<impl IntoResponse, GlobalError> {
    if payload.email.is_empty() || payload.password.is_empty() || payload.nickname.is_empty() {
        return Err(AuthError::MissingCredential.into());
    }
    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(AuthError::InvalidCredential.into());  
    }

    let qr = entity::docuser::Entity::find()
        .filter(entity::docuser::Column::Email.eq(payload.email.clone()))
        .all(&state.db_conn)
        .await?;
    if qr.len() >= 1 {
        return Err(AuthError::DuplicateEmail.into());
    }
    
    let password_hash = module::password::create_hash(&payload.password[..].as_bytes()).map_err(|err| AuthError::from(err))?;
    let new_user = entity::docuser::ActiveModel {
        email: Set(payload.email), 
        nickname: Set(payload.nickname),
        hash: Set(password_hash),
        ..Default::default()
    };
    let insert_result = entity::docuser::Entity::insert(new_user).exec(&state.db_conn).await?;
    Ok(())

}
async fn issue(State(state): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, Json(payload): Json<IssuePayload> ) -> Result<impl IntoResponse, GlobalError> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredential.into());
    }
    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(AuthError::InvalidCredential.into());  
    }

    let qr = entity::docuser::Entity::find()
        .filter(entity::docuser::Column::Email.eq(payload.email.clone()))
        .one(&state.db_conn)
        .await?;
    if qr.is_none() {
        return Err(AuthError::InvalidCredential.into());
    }
    let qr = qr.unwrap();
    
    verify_password(&qr.hash, &payload.password[..].as_bytes()).map_err(|err| AuthError::from(err))?;

    let access_claims = Claims {
        iat: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(10)).timestamp(),
        iss: "docuvault".to_owned(),
        user_id: qr.id,
        token_typ: "access".to_owned(),
    };     
    let access_token = encode(&Header::default(), &access_claims, &ACCESS_KEYS.encoding).map_err(|err| AuthError::from(err))?;

    let refresh_claims = Claims {
        iat: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::days(3)).timestamp(),
        iss: "docuvault".to_owned(),
        user_id: qr.id,
        token_typ: "refresh".to_owned(),
    };
    let refresh_token = encode(&Header::default(), &refresh_claims, &REFRESH_KEYS.encoding).map_err(|err|AuthError::from(err))?;
    
    
    let mut schema =  TokenPair::new(RedisSchemaHeader {
        key: access_token.clone(),
        expire_at: Some(access_claims.exp as usize),
        con: state.redis_conn.clone(),
    });
    schema.set_refresh_token(refresh_token.clone()).flush().await?;

    let mut schema = Refresh::new(RedisSchemaHeader {
        key: refresh_token.clone(),
        expire_at: Some(refresh_claims.exp as usize),
        con: state.redis_conn.clone(),
    });
    schema.set_ip(addr.to_string()).flush().await?;
    
    Ok(Json(IssueResponse{
        access_token,
        refresh_token,
    }))
}
async fn disconnect(State(state): State<AppState>, TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>, claims: Claims) -> Result<impl IntoResponse, GlobalError> {
    let mut schema = BlackList::new(RedisSchemaHeader {
        key: bearer.token().to_string(),
        expire_at: Some(claims.exp as usize),
        con: state.redis_conn.clone(),
    });
    schema.set_status(true).flush().await?;

    let mut token_pair_schema = TokenPair::new(RedisSchemaHeader {
        key: bearer.token().to_string(),
        expire_at: None,
        con: state.redis_conn.clone(),
    });
    token_pair_schema.get_refresh_token().await?;

    if token_pair_schema.refresh_token.is_some() {
        let mut schema = Refresh::new(RedisSchemaHeader {
            key: token_pair_schema.refresh_token.clone().unwrap(),
            expire_at: None,
            con: state.redis_conn.clone(),
        });
        schema.del_all().await?;
    }
    token_pair_schema.del_all().await?;

    Ok(()) 
}

async fn refresh(State(state): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>) -> Result<impl IntoResponse, GlobalError> {

    let mut refresh_schema = Refresh::new(RedisSchemaHeader {
        key: bearer.token().to_string(),
        expire_at: None,
        con: state.redis_conn.clone(),
    });
    refresh_schema.get_ip().await?;

    if refresh_schema.ip.is_none() {
        return Err(AuthError::InvalidToken.into());
    }

    let last_client_ip = refresh_schema.ip.clone().unwrap().chars().take_while(|&c| c!=':').collect::<String>();
    let current_client_ip = addr.to_string().chars().take_while(|&c| c!=':').collect::<String>();
    if last_client_ip != current_client_ip {
        dbg!("ip changed");
        refresh_schema.del_all().await?;
        return Err(AuthError::IpChanged.into());
    }
    
    

    let token_data = decode::<Claims>(bearer.token(), &REFRESH_KEYS.decoding, &Validation::default())
        .map_err(|err| {
            if err.into_kind() == ErrorKind::ExpiredSignature {
                AuthError::TokenExpired
            }
            else {
                AuthError::InvalidToken
            }
        })?;

    let claims = Claims {
        iat: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(10)).timestamp(),
        iss: "docuvault".to_owned(),
        user_id: token_data.claims.user_id,
        token_typ: "access".to_owned(),
    };     

    let access_token = encode(&Header::default(), &claims, &ACCESS_KEYS.encoding).map_err(|err|AuthError::from(err))?;

    let mut schema = TokenPair::new(RedisSchemaHeader {
        key: access_token.clone(),
        expire_at: Some(claims.exp as usize),
        con: state.redis_conn.clone(),
    });
    schema.set_refresh_token(bearer.token().to_string()).flush().await?;

    Ok(Json(RefreshResponse{
        access_token,
    }))
}
