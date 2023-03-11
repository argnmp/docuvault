use std::{env, net::ToSocketAddrs};

use apis::{upload::{UploadService, upload::upload_server::UploadServer}, download::{download::download_server::DownloadServer, DownloadService}, delete::{delete::delete_server::DeleteServer, DeleteService}};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use futures::future::join_all;
use sea_orm::DatabaseConnection;
use tonic::transport::Server;

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

async fn serve(state: AppState, port: u16) -> Result<(), Box<dyn std::error::Error>>{
    let upload_service = UploadService {
        state: state.clone(),
    };
    let donwload_service = DownloadService {
        state: state.clone(),
    };
    let delete_service = DeleteService {
        state: state.clone(),
    };

    let server_addr = env::var("SERVER_ADDR").expect("server addr is not set");

    let server_addr = format!("{}:{}",server_addr,port).to_socket_addrs().unwrap().next().unwrap();
    dbg!(&server_addr);
    Server::builder()
        .add_service(UploadServer::new(upload_service))
        .add_service(DownloadServer::new(donwload_service))
        .add_service(DeleteServer::new(delete_service))
        .serve(server_addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let local_instance_num = env::var("LOCAL_INSTANCE_NUM").expect("local instance number is not set").parse::<i32>().expect("local instance number is not an integer");
    let state = AppState {
        db_conn: db::postgres_connect().await,
        redis_conn: db::redis_connect().await,
    };
    let servers = (0..local_instance_num).map(|i|serve(state.clone(), (9000+i) as u16)).collect::<Vec<_>>();
    join_all(servers).await;
    dbg!("exit");
    Ok(())

}
