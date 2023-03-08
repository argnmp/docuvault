use tonic::{Response, Status, Request};

use crate::{AppState, module::obj_hash::hash_to_limit};

use self::download::{delete_server::Delete, DeleteRequest, DeleteResponse, delete_client::DeleteClient};


pub mod download {
    use tonic::include_proto;
    include_proto!("delete");
}

pub struct DeleteService {
    pub state: AppState,
}

#[tonic::async_trait]
impl Delete for DeleteService {
    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<DeleteResponse>, Status> {
        let req = request.into_inner();
        
        let mut classified_obj_ids: Vec<Vec<String>> = vec![Vec::new(); self.state.file_server_num as usize];
        for object_id in req.object_ids {
            let dest = hash_to_limit(self.state.file_server_num as u64, &object_id);
            classified_obj_ids[dest].push(object_id);
        }

        for (idx, &addr) in self.state.file_server_addr.lock().await.iter().enumerate(){
            let mut delete_client = DeleteClient::connect(addr).await.unwrap();
            let _ = delete_client.delete(Request::new(DeleteRequest { object_ids: classified_obj_ids[idx].clone() })).await?;
        }


        return Ok(Response::new(DeleteResponse { msg: "delete succeeded".to_owned() }));
    }
}


