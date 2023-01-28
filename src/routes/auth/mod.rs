use axum::{Router, routing::get, response::{Html, IntoResponse}, extract::State};
use sea_orm::EntityTrait;

use crate::AppState;
use crate::entity;

mod error;

pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/issue", get(issue))
        .with_state(shared_state)
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    dbg!(entity::docorg::Entity::find_by_id(1).one(&state.db_conn).await);
    Html("welcome to auth index")
}

async fn issue(State(state): State<AppState>) -> Result<impl IntoResponse, error::AuthError> {
    if 2%2==0 {
        return Err(error::AuthError::MissingCredential) 
    }
    Ok(Html("success!"))
}