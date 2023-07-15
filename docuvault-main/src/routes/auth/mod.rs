use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::{Router, routing::{get, post}, response::{Html, IntoResponse}, extract::{State, ConnectInfo}, Json, middleware::{from_extractor, from_extractor_with_state}, TypedHeader, headers::{Authorization, authorization::Bearer}, http::{Method, header}};
use jsonwebtoken::{encode, Header, decode, Validation, errors::ErrorKind};
use sea_orm::{entity::*, query::*};
use regex::Regex;
use serde_json::json;
use redis::{AsyncCommands};
use tower_http::cors::{CorsLayer, Any};


use crate::{db::schema::redis::{TokenPair, RedisSchemaHeader, Refresh, BlackList}, common::object::ServiceState};
use crate::AppState;
use crate::entity;
use crate::middleware::guard::Authenticate;

pub mod error;
use error::*;
pub mod object;
use object::*;
pub mod service;
use service::*;
pub mod constant;
use constant::*;

use self::module::password::verify_password;

use super::error::GlobalError;

mod module;


pub fn create_router(shared_state: AppState) -> Router {
    let service_state: ServiceState<AuthService> = ServiceState{
        global_state: shared_state.clone(),
        service: Arc::new(AuthService::new(shared_state.clone())),
    };
    
    Router::new()
        .route("/protected", get(protected))
        .route("/disconnect", get(disconnect))
        .route_layer(from_extractor_with_state::<Authenticate, ServiceState<AuthService>>(service_state.clone()))
        .route("/", get(index))
        .route("/register", post(register))
        .route("/issue", post(issue))
        .route("/refresh", get(refresh))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::POST])
                .allow_headers([header::CONTENT_TYPE])
            )
        .route("/test", get(test))
        .with_state(service_state)
    
}
async fn test(State(state): State<ServiceState<AuthService>>) -> impl IntoResponse {
    let qr = state.service.find_user("kim@kim.com").await.unwrap();
    Html("hello this is state")
}

async fn protected() -> impl IntoResponse {
    Json(json!({"msg": "you've got access"}))
}
async fn index(State(state): State<ServiceState<AuthService>>) -> impl IntoResponse {
    Html("welcome to auth index")
}

async fn register(State(state): State<ServiceState<AuthService>>, Json(payload): Json<RegisterPayload>) -> Result<impl IntoResponse, GlobalError> {
    if payload.email.is_empty() || payload.password.is_empty() || payload.nickname.is_empty() {
        return Err(AuthError::MissingCredential.into());
    }
    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(AuthError::InvalidCredential.into());  
    }

    let qr = match state.service.find_users(&payload.email).await? {
        Some(qr) => qr,
        None => return Err(AuthError::DuplicateEmail.into()),
    };
    
    let password_hash = module::password::create_hash(&payload.password[..].as_bytes()).map_err(|err| AuthError::from(err))?;
    let new_user = entity::docuser::ActiveModel {
        email: Set(payload.email), 
        nickname: Set(payload.nickname),
        hash: Set(password_hash),
        ..Default::default()
    };
    let insert_result = entity::docuser::Entity::insert(new_user).exec(&state.global_state.db_conn).await?;
    Ok(())

}
async fn issue(State(state): State<ServiceState<AuthService>>, ConnectInfo(addr): ConnectInfo<SocketAddr>, Json(payload): Json<IssuePayload> ) -> Result<impl IntoResponse, GlobalError> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AuthError::MissingCredential.into());
    }
    let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(AuthError::InvalidCredential.into());  
    }
    
    let qr  = match state.service.find_user(&payload.email).await? {
        Some(qr) => qr,
        None => return Err(AuthError::InvalidCredential.into()),
    };
    
    verify_password(&qr.hash, &payload.password[..].as_bytes()).map_err(|err| AuthError::from(err))?;

    let access_token = state.service.issue_access_token(qr.id).await?;
    let refresh_token = state.service.issue_refresh_token(qr.id).await?;
    state.service.set_tokenpair(&access_token, &refresh_token).await?;
    state.service.set_refresh(&refresh_token, addr.to_string()).await?;
    
    Ok(Json(IssueResponse{
        access_token,
        refresh_token,
    }))
}
async fn disconnect(State(state): State<ServiceState<AuthService>>, TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>, claims: Claims) -> Result<impl IntoResponse, GlobalError> {
    state.service.set_redis_blacklist(bearer.token(), claims.exp as usize);
    state.service.disable_auth(bearer.token());
    Ok(()) 
}

async fn refresh(State(state): State<ServiceState<AuthService>>, ConnectInfo(addr): ConnectInfo<SocketAddr>, TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>) -> Result<impl IntoResponse, GlobalError> {

    let refresh_ip = state.service.get_refresh_ip(bearer.token()).await?;
    if refresh_ip.is_none() {
        return Err(AuthError::InvalidToken.into());
    }

    let last_client_ip = refresh_ip.unwrap().chars().take_while(|&c| c!=':').collect::<String>();
    let current_client_ip = addr.to_string().chars().take_while(|&c| c!=':').collect::<String>();
    if last_client_ip != current_client_ip {
        dbg!("ip changed");
        state.service.remove_refresh_record(bearer.token()).await?;
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


    let access_token = state.service.issue_access_token(token_data.claims.user_id).await?;

    // needs refactoring / same as length of access_token
    state.service.set_tokenpair(&access_token, bearer.token()).await?;

    Ok(Json(RefreshResponse{
        access_token,
    }))
}
