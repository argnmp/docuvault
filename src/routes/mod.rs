use axum::{Router, response::{Html, IntoResponse}, extract::State, routing::get};
use tower_http::trace::TraceLayer;

use crate::AppState;

mod error;
mod auth;

pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .with_state(shared_state.clone())
        .nest("/auth", auth::create_router(shared_state.clone()))
        .layer(TraceLayer::new_for_http())
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    Html("welcome to docuvault")
}
