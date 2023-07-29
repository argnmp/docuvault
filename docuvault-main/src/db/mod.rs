use std::{env, time::Duration, sync::Arc};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::Client;
use sea_orm::{Database, DatabaseConnection, ConnectOptions};

pub async fn postgres_connect() -> DatabaseConnection {
    let database_url = env::var("DATABASE_URL").expect("database url is not set");
    dbg!(&database_url);
    let mut opt = ConnectOptions::new(database_url);
    opt
        .max_connections(100)
        .min_connections(5)    
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    Database::connect(opt).await.expect("error establishing db pool")

}

pub mod schema;
pub async fn redis_connect() -> Pool<RedisConnectionManager> {
    let redis_url = env::var("REDIS_URL").expect("redis url is not set");
    dbg!(&redis_url);
    let manager = RedisConnectionManager::new(redis_url).unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();
    pool
}

