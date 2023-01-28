use std::{env, time::Duration};
use sea_orm::{Database, DatabaseConnection, ConnectOptions};

pub async fn connect() -> DatabaseConnection {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("database url is not set");
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