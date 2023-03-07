use sha2::{Sha256, Digest};
use tokio::io::AsyncWriteExt;
use tokio::{fs::create_dir_all, fs::File};
use tonic::{Request, Response, Status};
use sea_orm::{entity::*, query::*};
use sea_orm::{Set, ActiveModelTrait, EntityTrait};
use upload::upload_server::Upload;

use crate::{AppState, db::schema::redis::{DocFile, RedisSchemaHeader}, entity, error::GlobalError};

use self::upload::{UploadRequest, UploadResponse, PreUploadRequest, PreUploadResponse};
pub mod upload {
    use tonic::include_proto;
    include_proto!("upload");
}

#[derive(Debug)]
pub struct UploadService {
    pub state: AppState,
}

#[tonic::async_trait]
impl Upload for UploadService {
    async fn pre_upload(&self, request: Request<PreUploadRequest>) -> Result<Response<PreUploadResponse>, Status> {
        let req = request.into_inner();

        let mut hasher = Sha256::new();
        hasher.update(&req.name[..]);
        hasher.update(&req.ftype[..]);
        hasher.update(chrono::Utc::now().timestamp().to_string());
        let hash = hasher.finalize();
        let hash_str = format!("{:x}", hash);

        let mut docfile: entity::docfile::ActiveModel = entity::docfile::ActiveModel {
            docorg_id: Set(None),
            docuser_id: Set(req.docuser_id),
            object_id: Set(hash_str.clone()),
            name: Set(req.name.clone()),
            ftype: Set(req.ftype.clone()),
            size: Set(req.size.clone() as i64),
            uri: Set(None),
            is_fixed: Set(false),
            status: Set(0),
            ..Default::default()
        }.insert(&self.state.db_conn).await.map_err(|err|{GlobalError::from(err)})?.into();

        let db_conn = self.state.db_conn.clone();
        let redis_conn = self.state.redis_conn.clone();
        let object_id = hash_str.clone();
        tokio::spawn(async move {
            let req = req;
            let file_name = req.name.clone(); 
            let file_path = format!("./{}/{}","files", object_id);
            //0: pending, 1: pass, 2: fail
            let mut status = 1;

            match create_dir_all(&file_path).await {
                Ok(_) => {},
                Err(e) => {
                    dbg!(e);
                    status = 2;
                }
            }
        
            let file_full_path = format!("{}/{}",&file_path, &file_name);
            match File::create(&file_full_path).await {
                Ok(mut f) => {
                    match f.write_all(&req.data[..]).await {
                        Ok(_) => {},
                        Err(e) => {
                            dbg!(e);
                            status = 2;
                        }
                    }
                },
                Err(e) => {
                    dbg!(e);
                    status = 2;
                }
            };

            docfile.uri = Set(Some(file_full_path));
            docfile.status = Set(status);

            docfile.update(&db_conn).await.expect("db connection failed");

            let mut file_schema = DocFile::new(RedisSchemaHeader{
                key: object_id.clone(),
                con: redis_conn,
                expire_at: Some((chrono::Utc::now()+chrono::Duration::minutes(10)).timestamp() as usize),
            });
            file_schema.set_name(req.name).set_ftype(req.ftype).set_data(req.data).set_size(req.size).flush().await.expect("redis landing failed");
        });
        

        return Ok(Response::new(PreUploadResponse {
            object_id: hash_str,
        }))
    }


    //upload is synchronized to api server
    async fn upload(&self, request: Request<UploadRequest>) -> Result<Response<UploadResponse>, Status> {
        let req = request.into_inner();
        let docfile = entity::docfile::Entity::find()
            .filter(entity::docfile::Column::ObjectId.eq(req.object_id))
            .one(&self.state.db_conn)
            .await.map_err(|err|{GlobalError::from(err)})?;

        if docfile.is_none() {
            return Err(GlobalError::ObjectNotExist.into());     
        }
        let mut docfile: entity::docfile::ActiveModel = docfile.unwrap().into();
        docfile.docorg_id = Set(Some(req.doc_id));
        docfile.is_fixed = Set(true);
        docfile.update(&self.state.db_conn).await.map_err(|err|{GlobalError::from(err)})?;
        
        return Ok(Response::new(UploadResponse {
            msg: "upload successful".to_string(),
        }))
    }
}
