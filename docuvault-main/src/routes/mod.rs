use std::env;

use axum::{Router, response::{Html, IntoResponse}, extract::State, routing::get};
use tower_http::trace::TraceLayer;

use crate::{AppState, modules::grpc::upload::upload_client::UploadClient};

pub mod error;
pub mod auth;
pub mod document;
pub mod resource;
pub mod file;

pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .with_state(shared_state.clone())
        .nest("/auth", auth::create_router(shared_state.clone()))
        .nest("/document", document::create_router(shared_state.clone()))
        .nest("/resource", resource::create_router(shared_state.clone()))
        .nest("/file", file::create_router(shared_state.clone()))
        .layer(TraceLayer::new_for_http())
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let file_proxy_addr = env::var("FILE_PROXY_ADDR").expect("file proxy addr is not set.");
    dbg!(&file_proxy_addr);
    let mut upload_client = UploadClient::connect(file_proxy_addr).await.unwrap();
    Html("welcome to docuvault")
}
