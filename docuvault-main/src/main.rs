#![allow(unused)]
use std::{net::SocketAddr, env};
use axum::extract::FromRef;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use migration::{Migrator, MigratorTrait};


mod entity;
mod db;
mod routes;
mod middleware;
mod bootstrap;
mod modules;


// database connection pool implements clone by internally using Arc
#[derive(Clone, Debug)]
pub struct AppState {
    db_conn: DatabaseConnection,
    redis_conn: Pool<RedisConnectionManager>
}
impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(input: &AppState) -> Self {
        input.db_conn.clone()
    } 
}
impl FromRef<AppState> for Pool<RedisConnectionManager> {
    fn from_ref(input: &AppState) -> Self {
        input.redis_conn.clone()
    }
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

    let db_conn    = db::postgres_connect().await;
    /*
     * migrate database
     */
    Migrator::up(&db_conn, None).await;

    let redis_conn = db::redis_connect().await;
    let state = AppState{
        db_conn,
        redis_conn 
    };

    
    let addr = SocketAddr::from(([0,0,0,0], 8000));
    
    bootstrap::bootstrap(state.clone()).await;
    


    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(routes::create_router(state).into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
