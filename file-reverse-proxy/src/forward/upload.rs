use sha2::{Sha256, Digest};
use tonic::{Request, Response, Status};

use crate::{AppState, module::obj_hash::hash_to_limit};

use self::upload::{upload_server::Upload, PreUploadRequest, PreUploadResponse, UploadRequest, UploadResponse, upload_client::UploadClient};

pub mod upload {
    use tonic::include_proto;

    include_proto!("upload");
}

pub struct UploadService {
    pub state: AppState,
}

#[tonic::async_trait]
impl Upload for UploadService {
    async fn pre_upload(&self, request: Request<PreUploadRequest>) -> Result<Response<PreUploadResponse>, Status> {
        let mut req = request.into_inner();

        let mut hasher = Sha256::new();
        hasher.update(&req.name[..]);
        hasher.update(&req.ftype[..]);
        hasher.update(chrono::Utc::now().timestamp().to_string());
        let hash = hasher.finalize();
        let hash_str = format!("{:x}", hash);

        /*
         * forward request
         */
        let addr = self.state.file_server_addr.lock().await[hash_to_limit(self.state.file_server_num as u64, &hash_str)];
        dbg!(&addr);
        req.object_id = Some(hash_str);
        let mut upload_client = UploadClient::connect(addr).await.unwrap();
        let res = upload_client.pre_upload(Request::new(req)).await?;

        return Ok(res);
    }


    //upload is synchronized to api server
    async fn upload(&self, request: Request<UploadRequest>) -> Result<Response<UploadResponse>, Status> {
        let req = request.into_inner();
        let addr = self.state.file_server_addr.lock().await[hash_to_limit(self.state.file_server_num as u64, &req.object_id)];

        let mut upload_client = UploadClient::connect(addr).await.unwrap();
        let res = upload_client.upload(Request::new(req)).await?;
        
        return Ok(res)
    }
}
