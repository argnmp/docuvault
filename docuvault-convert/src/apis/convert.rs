use std::{path::PathBuf, env};

use pandoc::{OutputKind, InputKind, PandocOutput, PandocError};
use tokio::{fs::{File, self}, io::AsyncWriteExt, io::AsyncBufRead};
use tonic::{Request, Response, Status};
use sea_orm::{entity::*, query::*};


use crate::{AppState, entity, error::GlobalError, grpc::upload::{upload_client::UploadClient, UploadRequest, PreUploadRequest}};

use self::convert::{convert_server::Convert, ConvertRequest, ConvertResponse};

pub mod convert {
    use tonic::include_proto;
    include_proto!("convert");
}

pub struct ConvertService {
    pub state: AppState,
}

fn extension<'a>(c_type: i32)->&'a str{
    match c_type {
        0 => "html",
        1 => "html",
        2 => "txt",
        3 => "docx",
        4 => "pdf",
        5 => "epub",
        6 => "json",
        _ => "",
    }
}
fn ftype<'a>(c_type: i32)->&'a str{
    match c_type {
        0 => "text/html",
        1 => "text/html",
        2 => "text/plain",
        3 => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        4 => "application/pdf",
        5 => "application/epub+zip",
        6 => "application/json",
        _ => "",
    }
}

fn pandoc_creation(req: &ConvertRequest) -> Result<PandocOutput, PandocError> {
    let mut pandoc = pandoc::new();
    pandoc.set_input(InputKind::Pipe(req.data.clone()));
    pandoc.set_output(OutputKind::File(PathBuf::from(format!("output.{}", extension(req.c_type)))));
    pandoc.execute() 
}

#[tonic::async_trait]
impl Convert for ConvertService {
    async fn convert(&self, request: Request<ConvertRequest>) -> Result<Response<ConvertResponse>, Status> {
        let req = request.into_inner();    
        
        let convertres = entity::convert::ActiveModel {
            docorg_id: Set(req.doc_id),
            c_type: Set(req.c_type), 
            data: Set(None),
            status: Set(0),
            ..Default::default()
        }.insert(&self.state.db_conn).await.map_err(|err|GlobalError::from(err));
        let mut convertres = match convertres {
            Ok(model) => model.into_active_model(),
            Err(e) => return Err(e.into()),
        };
        
        let db_conn = self.state.db_conn.clone();

        tokio::spawn(async move {
            match pandoc_creation(&req) {
                Ok(_) => {
                    match fs::read(format!("output.{}",extension(req.c_type))).await {
                        Ok(bytes) => {
                            let file_proxy_addr = env::var("FILE_PROXY_ADDR").expect("file proxy addr is not set.");
                            let mut upload_client = UploadClient::connect(file_proxy_addr).await.unwrap();

                            match upload_client.pre_upload(Request::new(PreUploadRequest{
                                name: format!("{}.{}",req.title,extension(req.c_type)), 
                                docuser_id: req.docuser_id,
                                ftype: ftype(req.c_type).to_owned(),
                                size: bytes.len() as u64,
                                data: bytes,
                                object_id: None,
                            })).await {
                                Ok(res) => {
                                    let res = res.into_inner();
                                    match upload_client.upload(Request::new(UploadRequest { object_id: res.object_id.clone(), doc_id: req.doc_id })).await {
                                        Ok(_) => {
                                            convertres.data = Set(Some(res.object_id));
                                            convertres.status = Set(1);
                                            convertres.update(&db_conn).await.unwrap();

                                        },
                                        Err(e) => {
                                            dbg!(e);
                                            convertres.status = Set(2);
                                            convertres.insert(&db_conn).await.unwrap();
                                        }
                                    }

                                },
                                Err(e) => {
                                    dbg!(e);
                                    convertres.status = Set(2);
                                    convertres.insert(&db_conn).await.unwrap();
                                }
                            }


                        },
                        Err(e) => {
                            dbg!(e);
                            convertres.status = Set(2);
                            convertres.insert(&db_conn).await.unwrap();
                        }
                    }
                }
                Err(e) => {
                    dbg!(e);
                    convertres.status = Set(2);
                    convertres.insert(&db_conn).await.unwrap();
                }

            }

        });

        return Ok(Response::new(ConvertResponse {
            msg: "hello".to_owned(),
        }));
    }
}


