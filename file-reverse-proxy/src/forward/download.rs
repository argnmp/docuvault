use tonic::{Response, Status, Request};

use crate::{AppState, module::obj_hash::hash_to_limit};

use self::download::{DownloadRequest, DownloadResponse, download_server::Download, download_client::DownloadClient};

pub mod download {
    use tonic::include_proto;
    include_proto!("download");
}

pub struct DownloadService {
    pub state: AppState,
}

#[tonic::async_trait]
impl Download for DownloadService {
    async fn download(&self, request: Request<DownloadRequest>) -> Result<Response<DownloadResponse>, Status> {
        let req = request.into_inner();
        let addr = self.state.file_server_addr.lock().await[hash_to_limit(self.state.file_server_num as u64, &req.object_id)];

        let mut download_client = DownloadClient::connect(addr).await.unwrap();
        let res = download_client.download(Request::new(req)).await?;

        return Ok(res);
    }
}

