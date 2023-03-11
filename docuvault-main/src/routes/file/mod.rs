use std::env;

use axum::{Router, extract::{State, Query, Multipart, DefaultBodyLimit, Path, BodyStream}, routing::{get, post}, response::IntoResponse, body::Bytes, headers::ContentType, http::{header, Method, HeaderValue}, Json,};
use sha2::{Sha256, Digest};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use tonic::Request;
use tower_http::cors::CorsLayer;
use futures::StreamExt;
use futures_util::stream;

use crate::{AppState, modules::grpc::{voting::{voting_client::VotingClient, VotingRequest}, upload::{upload_client::UploadClient, PreUploadRequest, UploadRequest}, download::{download_client::DownloadClient, DownloadRequest}}, db::schema::redis::{File, RedisSchemaHeader}};

pub mod error;
use error::*;
mod object;
use object::*;

use super::{error::GlobalError, auth::object::Claims};

pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/upload", post(upload))
        .route("/uploadfix", post(uploadfix))
        .route("/:object_id", get(download))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::OPTIONS, Method::POST])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .allow_credentials(true)
            )
        .with_state(shared_state)
        .layer(DefaultBodyLimit::max(1024*1024*1024*1024))
}


//preupload
async fn upload(State(state): State<AppState>, claims: Claims, mut multipart: Multipart) -> Result<impl IntoResponse, GlobalError> {
    let file_proxy_addr = env::var("FILE_PROXY_ADDR").expect("file proxy addr is not set.");
    let mut upload_client = UploadClient::connect(file_proxy_addr).await.unwrap();
    let mut object_ids = vec![];
    #[derive(Serialize)]
    struct Resource {
        name: String,
        ftype: String,
        object_id: String,
    }
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let ftype = field.content_type().unwrap().to_string();
        let name = field.file_name().unwrap().to_string();
        let data = Vec::from(&field.bytes().await.unwrap()[..]);
        let size = data.len();

        let request = tonic::Request::new(PreUploadRequest{
            name: name.clone(),
            ftype: ftype.clone(),
            size: size as u64,
            docuser_id: claims.user_id,
            object_id: None,
            data,
        });
        let response = upload_client.pre_upload(request).await.unwrap();
        object_ids.push(Resource{
            name,
            ftype,
            object_id: response.into_inner().object_id,
        });
    }
    Ok(Json(object_ids))
}
#[derive(Deserialize)]
struct UploadfixPayload{
    doc_id: i32,
    object_id: String,
}
async fn uploadfix(State(state): State<AppState>, Json(payload): Json<UploadfixPayload>) -> Result<impl IntoResponse, GlobalError> {
    let file_proxy_addr = env::var("FILE_PROXY_ADDR").expect("file proxy addr is not set.");
    let mut upload_client = UploadClient::connect(file_proxy_addr).await.unwrap();
    let res = upload_client.upload(UploadRequest {
        doc_id: payload.doc_id,
        object_id: payload.object_id,
    }).await?;
    Ok(res.into_inner().msg)
}

async fn download(State(state): State<AppState>, Path(object_id): Path<String>) -> Result<impl IntoResponse, GlobalError>{
    let file_proxy_addr = env::var("FILE_PROXY_ADDR").expect("file proxy addr is not set.");
    let mut client = DownloadClient::connect(file_proxy_addr).await?;
    
    let req = tonic::Request::new(DownloadRequest{
        object_id: object_id.clone(),
    });
    let res = client.download(req).await?.into_inner();
    
    Ok(([(header::CONTENT_TYPE, res.ftype), (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", res.name))], res.data))
}
