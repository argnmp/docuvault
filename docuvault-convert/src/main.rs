use std::{env, net::ToSocketAddrs};

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;

mod db;
mod entity;
mod apis;
mod grpc;
mod error;
mod test;

use apis::convert::{ConvertService, convert::convert_server::ConvertServer};
use tonic::transport::Server;

#[derive(Clone, Debug)]
pub struct AppState {
    db_conn: DatabaseConnection,
    redis_conn: Pool<RedisConnectionManager>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let state = AppState {
        db_conn: db::postgres_connect().await,
        redis_conn: db::redis_connect().await,
    };
    let convert_service = ConvertService {
        state: state.clone(),
    };
    let server_addr = env::var("SERVER_ADDR").expect("server addr is not set");
    let server_addr = format!("{}:{}",server_addr,7000).to_socket_addrs().unwrap().next().unwrap();
    dbg!(&server_addr);
    Server::builder()
        .add_service(ConvertServer::new(convert_service))
        .serve(server_addr)
        .await?;

    Ok(())
}
