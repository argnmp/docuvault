use std::net::SocketAddr;
use sea_orm::DatabaseConnection;
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


mod entity;
mod db;
mod routes;

#[derive(Clone)]
pub struct AppState {
    db_conn: DatabaseConnection
}


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "docuvault=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_conn = db::connect().await;
    let state = AppState{
        db_conn,
    };
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(routes::create_router(state).into_make_service())
        .await
        .unwrap();
}
