use tokio::fs;
use tonic::{Request, Response, Status};
use sea_orm::{entity::*, query::*};
use sea_orm::{Set, ActiveModelTrait, EntityTrait};

use crate::{AppState, db::schema::redis::{DocFile, RedisSchemaHeader}, entity};

use self::delete::{delete_server::Delete, DeleteRequest, DeleteResponse};

pub mod delete{
    use tonic::include_proto;
    include_proto!("delete");
}

#[derive(Debug)]
pub struct DeleteService {
    pub state: AppState,
}

#[tonic::async_trait]
impl Delete for DeleteService {
    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<DeleteResponse>, Status> {
        let req = request.into_inner();
        for object_id in req.object_ids {
            delete_callback(self.state.clone(), object_id);
        }
        return Ok(Response::new(DeleteResponse { msg: "delete operation accepted".to_string() }));
    }
}

fn delete_callback(state: AppState, object_id: String) {
    tokio::spawn(async move {
        let mut file_schema = DocFile::new(RedisSchemaHeader{
            key: object_id.clone(),
            con: state.redis_conn.clone(),
            expire_at: None,
        });
            match file_schema.del_all().await {
                Ok(_) => {},
                Err(_) => {},
            };

        let file_path = format!("./{}/{}","files", &object_id);
        match fs::remove_dir_all(file_path).await {
            Ok(()) => {},
            Err(e) => {
                dbg!(e);
            },
        };

        /*
         * delete entry in db
         * do not care whether file alives in db
         */

        // let _ = entity::docfile::Entity::delete_many()
            // .filter(entity::docfile::Column::ObjectId.eq(object_id))
            // .exec(&state.db_conn)
            // .await;
    });
}

