use tonic::{Request, Response, Status};
use sea_orm::{entity::*, query::*};
use sea_orm::EntityTrait;

use crate::{AppState, db::schema::redis::{DocFile, RedisSchemaHeader}, entity, error::GlobalError};

use self::download::{*, download_server::*};

pub mod download {
    use tonic::include_proto;
    include_proto!("download");
}

#[derive(Debug)]
pub struct DownloadService {
    pub state: AppState,
}

#[tonic::async_trait]
impl Download for DownloadService {
    async fn download(&self, request: Request<DownloadRequest>) -> Result<Response<DownloadResponse>, Status> {
        let req = request.into_inner();
        let key = req.object_id;
        let mut file_schema = DocFile::new(RedisSchemaHeader{
            key: key.clone(),
            con: self.state.redis_conn.clone(),
            expire_at: None,
        });
        file_schema.get_all().await?;
        
        if file_schema.name.is_some() {
            return Ok(Response::new(DownloadResponse {
                name: file_schema.name.unwrap(),
                ftype: file_schema.ftype.unwrap(),
                size: file_schema.size.unwrap(),
                data: file_schema.data.unwrap(),
            }));
        }

        let docfile = entity::docfile::Entity::find()
            .filter(entity::docfile::Column::ObjectId.eq(key))
            .one(&self.state.db_conn)
            .await.map_err(|err|GlobalError::from(err))?;
        if docfile.is_none() {
            return Err(GlobalError::ObjectNotExist.into());
        }
        let docfile = docfile.unwrap();

        let data = tokio::fs::read(docfile.uri.unwrap()).await.map_err(|err|GlobalError::from(err))?; 
        
        //important!!: redis schema must be changed to use reference
        file_schema.set_name(docfile.name.clone()).set_size(docfile.size as u64).set_ftype(docfile.ftype.clone()).set_data(data.clone()).flush().await?;
        Ok(Response::new(DownloadResponse {
            name: docfile.name,
            ftype: docfile.ftype,
            size: docfile.size as u64,
            data,
        }))
    }
}


