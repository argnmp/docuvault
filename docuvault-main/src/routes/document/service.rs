use std::sync::{Arc, Mutex};
use sea_orm::{entity::*, query::*, FromQueryResult};
use crate::{AppState, modules::{redis::redis_does_docuser_have_scope, markdown::get_title, tag::{TagSetModule, application::port::input::TagSetUseCase, domain::entity::tag::Tag}}, routes::error::GlobalError, entity::{self, docorg::ActiveModel}};

use super::{object::{DocumentStatus, PendingCreatePayload, PendingCreateResponse, CreatePayload}, error::DocumentError};

#[derive(Clone, Debug)]
pub struct DocumentService{
    state: AppState,
    tag_module: Arc<TagSetModule>,
}
impl DocumentService {
    pub fn new(shared_state: AppState) -> Self{
        Self {
            state: shared_state.clone(),
            tag_module: Arc::new(TagSetModule::new(shared_state.db_conn)),
        }
    }
    
    pub async fn check_user_has_scope(&self, docuser_id: i32, scope_ids: &[i32]) -> Result<(), GlobalError>{
        redis_does_docuser_have_scope(self.state.clone(), scope_ids, docuser_id).await?;
        Ok(())
    }

    pub async fn get_pending_document_records(&self, docuser_id: i32) -> Result<Vec<entity::docorg::Model>, GlobalError> {
        let document = entity::docorg::Entity::find()
            .filter(
                Condition::all()
                    .add(entity::docorg::Column::DocuserId.eq(docuser_id))
                    .add(entity::docorg::Column::Status.eq(DocumentStatus::PENDING as i32))
                )
            .all(&self.state.db_conn).await?;
        Ok(document)
    }


    pub async fn create_or_get_pending_document(&self, docuser_id: i32, payload: PendingCreatePayload) -> Result<PendingCreateResponse, GlobalError>{
        let document = self.get_pending_document_records(docuser_id).await?;
        let res: PendingCreateResponse;
        match document.len() {
            0 => {
                let new_document = entity::docorg::ActiveModel {
                    title: Set("".to_string()),
                    raw: Set(payload.raw.clone()),
                    docuser_id: Set(docuser_id),
                    status: Set(DocumentStatus::PENDING as i32),
                    ..Default::default()
                };
                entity::docorg::Entity::insert(new_document).exec(&self.state.db_conn).await?;
                res = PendingCreateResponse {
                    exists: false,
                    raw: payload.raw,
                }
            },
            1 => {
                res = PendingCreateResponse {
                    exists: true,
                    raw: document[0].raw.clone(),
                } 

            },
            _ => {
                return Err(GlobalError::InternalServerError);
            }
        }
                
        Ok(res)
    }
    pub async fn overwrite_pending_document(&self, docuser_id: i32, payload: PendingCreatePayload) -> Result<PendingCreateResponse, GlobalError>{
        let document = self.get_pending_document_records(docuser_id).await?;
        let res: PendingCreateResponse;
        match document.len() {
            0 => {
                return Err(DocumentError::DocumentNotExist.into());
            },
            1 => {
                let mut updated_document: ActiveModel = document[0].clone().into();
                updated_document.raw = Set(payload.raw.clone());  
                updated_document.update(&self.state.db_conn).await?;
                res = PendingCreateResponse {
                    exists: true,
                    raw: payload.raw,
                } 
            },
            _ => {
                return Err(GlobalError::InternalServerError);
            }
        }
        Ok(res)
    }
    pub async fn complete_pending_document(&self, docuser_id: i32, raw: &str) -> Result<i32, GlobalError>{
        let document = self.get_pending_document_records(docuser_id).await?;
        let res: i32;
        match document.len() {
            0 => {
                return Err(DocumentError::DocumentNotExist.into());
            },
            1 => {
                let mut updated_document: ActiveModel = document[0].clone().into();
                updated_document.title = Set(get_title(raw));
                updated_document.raw = Set(raw.to_string());  
                updated_document.status = Set(DocumentStatus::CREATED as i32);
                let updated_document = updated_document.update(&self.state.db_conn).await?;
                res = updated_document.id;
            },
            _ => {
                return Err(GlobalError::InternalServerError);
            }
        }
        Ok(res)
    }
    pub async fn test_tag(&self) -> Result<(), GlobalError>{
        let tag_service = self.tag_module.get_service();
        tag_service.add("sample tag".to_string()).await?;
        Ok(())
    }
}

