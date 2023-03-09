use std::{env, sync::Arc, net::ToSocketAddrs};
use tokio::sync::Mutex;

use forward::{upload::{upload::upload_server::UploadServer, UploadService}, download::{DownloadService, download::download_server::DownloadServer}, delete::{DeleteService, download::delete_server::DeleteServer}};
use tonic::transport::Server;

mod module;
mod forward;
mod error;

#[derive(Clone)]
pub struct AppState {
    pub file_server_num: i32,
    pub file_server_addr: Arc<Mutex<Vec<&'static str>>>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let server_addr = env::var("SERVER_ADDR").expect("server addr is not set");

    let address = format!("{}:8080",server_addr).to_socket_addrs().unwrap().next().unwrap();

    let file_server_num = env::var("FILE_SERVER_NUM").expect("file server number is not set").parse::<i32>().expect("file server number is not an integer number");
    let mut file_server_addr = vec![];
    for i in 1..=file_server_num {
        let addr: &'static str = Box::leak(env::var(format!("FILE_SERVER_{}_ADDR", i)).expect(&format!("file server {} address is not set", i)).into_boxed_str());
        file_server_addr.push(addr);
    }

    let state = AppState {
        file_server_num,
        file_server_addr: Arc::new(Mutex::new(file_server_addr)),
    };

    let upload_service = UploadService {
        state: state.clone(),
    };
    let donwload_service = DownloadService {
        state: state.clone(),
    };
    let delete_service = DeleteService {
        state: state.clone(),
    };
    Server::builder()
        .add_service(UploadServer::new(upload_service))
        .add_service(DownloadServer::new(donwload_service))
        .add_service(DeleteServer::new(delete_service))
        .serve(address)
        .await?;
    Ok(())
}

