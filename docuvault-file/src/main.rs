use apis::{UploadService, upload::upload_server::UploadServer, download::download_server::DownloadServer, DownloadService};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;
use tonic::{Request, Status, Response, transport::Server};

pub mod voting {
    use tonic::include_proto;
    include_proto!("voting");
}

mod entity;
mod db;
mod apis;
mod error;

#[derive(Clone, Debug)]
pub struct AppState {
    db_conn: DatabaseConnection,
    redis_conn: Pool<RedisConnectionManager>,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let address = "[::1]:8080".parse().unwrap();
    let state = AppState {
        db_conn: db::postgres_connect().await,
        redis_conn: db::redis_connect().await,
    };

    let upload_service = UploadService {
        state: state.clone(),
    };
    let donwload_service = DownloadService {
        state: state.clone(),
    };

    Server::builder()
        .add_service(UploadServer::new(upload_service))
        .add_service(DownloadServer::new(donwload_service))
        .serve(address)
        .await?;
    Ok(())
}
