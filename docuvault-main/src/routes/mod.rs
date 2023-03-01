use axum::{Router, response::{Html, IntoResponse}, extract::State, routing::get};
use tower_http::trace::TraceLayer;

use crate::AppState;

pub mod error;
pub mod auth;
pub mod document;
pub mod resource;

pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .with_state(shared_state.clone())
        .nest("/auth", auth::create_router(shared_state.clone()))
        .nest("/document", document::create_router(shared_state.clone()))
        .nest("/resource", resource::create_router(shared_state.clone()))
        .layer(TraceLayer::new_for_http())
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    Html("welcome to docuvault")
}
