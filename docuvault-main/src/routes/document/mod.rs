use std::borrow::BorrowMut;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::Path;
use axum::http::{Method, header, HeaderValue};
use axum::response::Html;
use axum::routing::{get, options};
use axum::{Router, extract::State, Json, response::IntoResponse, routing::post, middleware::from_extractor_with_state};
use comrak::ComrakOptions;
use jsonwebtoken::{encode, Header};
use redis::AsyncCommands;
use sea_orm::{entity::*, query::*, FromQueryResult};
use serde::Serialize;
use tower_http::cors::{CorsLayer, Any};
use crate::db::macros::RedisSchemaHeader;
use crate::modules::background::conversion::{self, convert_to_html};
use crate::modules::markdown::get_title;
use crate::modules::redis::redis_does_docuser_have_scope;
use crate::{AppState, entity, redis_schema};


pub mod error;
use error::*;
mod object;
use object::*;


use super::error::GlobalError;
use super::auth::object::Claims;
use super::auth::object::Claims as Authenticate;
use super::resource::error::ResourceError;

pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/get_update_resource/:doc_id", post(get_update_resource))
        .route("/delete", post(delete))
        .route("/update", post(update))
        .route("/create", post(create))
        .route("/publish", post(publish))
        .route("/", post(get_document))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::OPTIONS, Method::POST])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .allow_credentials(true)
            )
        .with_state(shared_state)
}
async fn create(State(state): State<AppState>, claims: Claims, Json(payload): Json<CreatePayload>) -> Result<impl IntoResponse, GlobalError>{
    /*
     * check user has scope
     */
    redis_does_docuser_have_scope(state.clone(), &payload.scope_ids[..], claims.user_id).await?;

    /*
     * insert new document(docorg)
     */

    let cloned_state = state.clone();
    let mut convertres = Arc::new(Mutex::new(None));
    let cloned_convertres = convertres.clone();
    let cloned_payload = payload.clone();
    
    state.db_conn.clone().transaction::<_, (), GlobalError>(|txn|{
        Box::pin(async move {
            let state = cloned_state;
            let convertres = cloned_convertres;
            let payload = cloned_payload;
            /*
             * create document
             */
            let new_doc = entity::docorg::ActiveModel {
                title: Set(get_title(&payload.raw.clone())),
                raw: Set(payload.raw.clone()), 
                docuser_id: Set(claims.user_id),
                status: Set(1),
                ..Default::default()
            };
            let docres = entity::docorg::Entity::insert(new_doc).exec(txn).await?;


            /*
             * create pending convert to html
             */

            let new_convert = entity::convert::ActiveModel {
                docorg_id: Set(docres.last_insert_id),
                c_type: Set(0),
                status: Set(0),
                ..Default::default()
            };
            *convertres.lock().unwrap() =  Some(entity::convert::Entity::insert(new_convert).exec(txn).await?.last_insert_id);


            /*
             * connect to scope
             */
            let scope_ids: Vec<_> = payload.scope_ids.iter().map(|&value|{
                entity::docorg_scope::ActiveModel {
                    docorg_id: Set(docres.last_insert_id),
                    scope_id: Set(value),
                    ..Default::default()
                }
            }).collect();
            let _ = entity::docorg_scope::Entity::insert_many(scope_ids).exec(txn).await?;

            /*
             * connect to tags
             */

            if payload.tags.len()>0 {
                let mut con = state.redis_conn.get().await?;
                let tags: std::collections::BTreeSet<String> = con.zrange("tags", 0, -1).await?;
                let document_tags = payload.tags.into_iter().map(|tag|tag.trim().to_lowercase().to_string()).collect::<std::collections::BTreeSet<_>>();

                let new_tags = document_tags.iter().filter(|tag| !tags.contains(&tag[..])).map(|tag| (0, tag.clone())).collect::<Vec<_>>();


                if new_tags.len() > 0{

                    let models = new_tags.iter().map(|(_, tag)| entity::tag::ActiveModel {
                        value: Set(tag.clone()),
                        ..Default::default()
                    }).collect::<Vec<_>>();

                    let res = entity::tag::Entity::insert_many(models).exec(txn).await?;

                }


                let mut cond = Condition::any();
                for tag in &document_tags {
                    cond = cond.add(entity::tag::Column::Value.eq(tag.clone()));

                }
                let res = entity::tag::Entity::find().filter(cond).all(txn).await?;

                let models = res.iter().map(|m|{
                    entity::docorg_tag::ActiveModel {
                        docorg_id: Set(docres.last_insert_id),
                        tag_id: Set(m.id),
                        ..Default::default()
                    }
                }).collect::<Vec<_>>();
                let res = entity::docorg_tag::Entity::insert_many(models).exec(txn).await?;


                //update tags last
                if new_tags.len() > 0{
                    let _:() = con.zadd_multiple("tags", &new_tags[..]).await?;
                }
            }

            /*
             * build sequence
             */

            if let Some(seq_id) = payload.seq_id {
                let seq = entity::sequence::Entity::find_by_id(seq_id)
                    .filter(entity::sequence::Column::DocuserId.eq(claims.user_id))
                    .one(&state.db_conn)
                    .await?;

                if seq.is_none() {
                    return Err(ResourceError::SequenceNotExist.into());  
                }

                #[derive(FromQueryResult, Serialize, Debug)]
                struct DocorgSeq  {
                    last_order: i32,
                };

                let seq = entity::docorg_sequence::Entity::find()
                    .filter(entity::docorg_sequence::Column::SequenceId.eq(seq_id))
                    .select_only()
                    .column_as(entity::docorg_sequence::Column::Order.max(), "last_order")
                    .group_by(entity::docorg_sequence::Column::SequenceId)
                    .into_model::<DocorgSeq>()
                    .one(txn)
                    .await?;

                let seq = match seq {
                    Some(seq) => seq,
                    None => DocorgSeq {
                        last_order: 0,
                    }
                };

                let docseq = entity::docorg_sequence::ActiveModel {
                    sequence_id: Set(seq_id),
                    docorg_id: Set(docres.last_insert_id),
                    order: Set(seq.last_order + 1),
                };

                docseq.insert(txn).await?;
            }
            Ok(())
        })
    }).await?; 

    let convert_id = (*convertres.lock().unwrap()).unwrap();
    conversion::convert_to_html(state, convert_id, payload.raw.clone());
    Ok(())
}

async fn get_update_resource(State(state): State<AppState>, claims: Claims, Path(doc_id): Path<i32>) -> Result<impl IntoResponse, GlobalError>{
    #[derive(FromQueryResult, Serialize, Debug)]
    struct Docs {
        id: i32,
        title: String,
        raw: String,
        status: i32,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
        scope_id: i32,
        tag_id: Option<i32>,
        tag_value: Option<String>,
        seq_id: Option<i32>,
    }
    let res = entity::docorg::Entity::find()
        .filter(entity::docorg::Column::Id.eq(doc_id))
        .filter(entity::docorg::Column::DocuserId.eq(claims.user_id))
        .join_rev(JoinType::LeftJoin, entity::docorg_scope::Relation::Docorg.def())
        .join_rev(JoinType::LeftJoin, entity::docorg_tag::Relation::Docorg.def())
        .join_rev(JoinType::LeftJoin, entity::tag::Entity::belongs_to(entity::docorg_tag::Entity)
                  .from(entity::tag::Column::Id)
                  .to(entity::docorg_tag::Column::TagId)
                  .into())
        .join_rev(JoinType::LeftJoin, entity::docorg_sequence::Relation::Docorg.def())
        .column_as(entity::docorg_scope::Column::ScopeId, "scope_id")
        .column_as(entity::docorg_tag::Column::TagId, "tag_id")
        .column_as(entity::tag::Column::Value, "tag_value")
        .column_as(entity::docorg_sequence::Column::SequenceId, "seq_id")
        .into_model::<Docs>()
        .all(&state.db_conn)
        .await?;
    
    if res.len() == 0 {
        return Err(DocumentError::DocumentNotExist.into());
    }

    #[derive(Serialize, Debug)]
    struct CompDocs {
        id: i32,
        scope_ids: BTreeSet<i32>,
        title: String,
        raw: String,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
        tags: BTreeSet<String>,
        seq_ids: BTreeSet<i32>,
    }
    let mut target: CompDocs = CompDocs {
        id: res[0].id,
        scope_ids: BTreeSet::new(),
        title: res[0].title.clone(),
        raw: res[0].raw.clone(),
        created_at: res[0].created_at,
        updated_at: res[0].updated_at,
        tags: BTreeSet::new(),
        seq_ids: BTreeSet::new(), 
    };
    
    for docs in res {
        target.scope_ids.insert(docs.scope_id);
        if let Some(tag_id) = docs.tag_id {
            target.tags.insert(docs.tag_value.unwrap());
        }
        if let Some(seq_id) = docs.seq_id {
            target.seq_ids.insert(seq_id);
        }
    } 

    Ok(Json(target))
}

async fn update(State(state): State<AppState>, claims: Claims, Json(payload): Json<UpdatePayload>) -> Result<impl IntoResponse, GlobalError>{

    redis_does_docuser_have_scope(state.clone(), &payload.scope_ids[..], claims.user_id).await?;

    state.db_conn.transaction::<_, (), GlobalError>(|txn|{
        let state = state.clone();
        let payload = payload.clone();
        Box::pin(async move {
            let document = entity::docorg::Entity::find()
                .filter(
                    Condition::all()
                        .add(entity::docorg::Column::Id.eq(payload.doc_id))
                        .add(entity::docorg::Column::DocuserId.eq(claims.user_id))
                    )
                .one(txn)
                .await?;
            if document.is_none() {
                return Err(DocumentError::DocumentNotExist.into());
            }
            let mut document: entity::docorg::ActiveModel = document.unwrap().into();
            // delete all connected scopes and tags
            let res = entity::docorg_scope::Entity::delete_many()
                .filter(entity::docorg_scope::Column::DocorgId.eq(payload.doc_id))
                .exec(txn)
                .await?;
            let res = entity::docorg_tag::Entity::delete_many()
                .filter(entity::docorg_tag::Column::DocorgId.eq(payload.doc_id))
                .exec(txn)
                .await?;

            // update target document;
            document.raw = Set(payload.raw.clone()); 
            document.title = Set(get_title(&payload.raw));
            document.updated_at = Set(chrono::Utc::now().naive_utc());
            document.update(txn).await?;

            let scope_ids: Vec<_> = payload.scope_ids.iter().map(|&value|{
                entity::docorg_scope::ActiveModel {
                    docorg_id: Set(payload.doc_id),
                    scope_id: Set(value),
                    ..Default::default()
                }
            }).collect();

            let _ = entity::docorg_scope::Entity::insert_many(scope_ids).exec(txn).await?;

            if payload.tags.len()>0 {
                let mut con = state.redis_conn.get().await?;
                let tags: std::collections::BTreeSet<String> = con.zrange("tags", 0, -1).await?;
                let document_tags = payload.tags.into_iter().map(|tag|tag.trim().to_lowercase().to_string()).collect::<std::collections::BTreeSet<_>>();

                let new_tags = document_tags.iter().filter(|tag| !tags.contains(&tag[..])).map(|tag| (0, tag.clone())).collect::<Vec<_>>();

                if new_tags.len() > 0{
                    let models = new_tags.iter().map(|(_, tag)| entity::tag::ActiveModel {
                        value: Set(tag.clone()),
                        ..Default::default()
                    }).collect::<Vec<_>>();

                    let res = entity::tag::Entity::insert_many(models).exec(txn).await?;
                }


                let mut cond = Condition::any();
                for tag in &document_tags {
                    cond = cond.add(entity::tag::Column::Value.eq(tag.clone()));

                }
                let res = entity::tag::Entity::find().filter(cond).all(txn).await?;

                let models = res.iter().map(|m|{
                    entity::docorg_tag::ActiveModel {
                        docorg_id: Set(payload.doc_id),
                        tag_id: Set(m.id),
                        ..Default::default()
                    }
                }).collect::<Vec<_>>();
                let res = entity::docorg_tag::Entity::insert_many(models).exec(txn).await?;

                //update tags last
                if new_tags.len() > 0{
                    let _:() = con.zadd_multiple("tags", &new_tags[..]).await?;
                }
            }
            /*
             * build sequence
             */

            if let Some(seq_id) = payload.seq_id {
                let seq = entity::sequence::Entity::find_by_id(seq_id)
                    .filter(entity::sequence::Column::DocuserId.eq(claims.user_id))
                    .one(&state.db_conn)
                    .await?;

                if seq.is_none() {
                    return Err(ResourceError::SequenceNotExist.into());  
                }

                let res = entity::docorg_sequence::Entity::delete_many()
                    .filter(entity::docorg_sequence::Column::DocorgId.eq(payload.doc_id))
                    .exec(txn)
                    .await?;

                #[derive(FromQueryResult, Serialize, Debug)]
                struct DocorgSeq  {
                    last_order: i32,
                };

                let seq = entity::docorg_sequence::Entity::find()
                    .filter(entity::docorg_sequence::Column::SequenceId.eq(seq_id))
                    .select_only()
                    .column_as(entity::docorg_sequence::Column::Order.max(), "last_order")
                    .group_by(entity::docorg_sequence::Column::SequenceId)
                    .into_model::<DocorgSeq>()
                    .one(txn)
                    .await?;

                let seq = match seq {
                    Some(seq) => seq,
                    None => DocorgSeq {
                        last_order: 0,
                    }
                };

                let docseq = entity::docorg_sequence::ActiveModel {
                    sequence_id: Set(seq_id),
                    docorg_id: Set(payload.doc_id),
                    order: Set(seq.last_order + 1),
                };

                docseq.insert(txn).await?;
            }
            else {
                let res = entity::docorg_sequence::Entity::delete_many()
                    .filter(entity::docorg_sequence::Column::DocorgId.eq(payload.doc_id))
                    .exec(txn)
                    .await?;

            }
            
            Ok(())
        })
    }).await?; 
    convert_to_html(state, (payload.doc_id, 0), payload.raw);
    Ok(())
}

async fn delete(State(state): State<AppState>, claims: Claims, Json(payload): Json<DeletePayload>) -> Result<impl IntoResponse, GlobalError>{
    let mut cond =  Condition::any();
    for doc_id in payload.doc_ids {
        cond = cond.add(entity::docorg::Column::Id.eq(doc_id));
    }
    let res = entity::docorg::Entity::delete_many()
        .filter(entity::docorg::Column::DocuserId.eq(claims.user_id))
        .filter(cond)
        .exec(&state.db_conn)
        .await?;

    Ok(())
}

async fn publish(State(state): State<AppState>, claims: Claims, Json(payload): Json<PublishPayload>) -> Result<impl IntoResponse, GlobalError>{
    let mut cond = Condition::any();
    for scope_id in payload.scope_ids {
        cond = cond.add(entity::docorg_scope::Column::ScopeId.eq(scope_id));
    }

    let res = entity::docorg_scope::Entity::find()
        .filter(
            Condition::all()
                .add(entity::docorg_scope::Column::DocorgId.eq(payload.doc_id))
            )
        .filter(cond)
        .join(JoinType::LeftJoin, entity::docorg_scope::Relation::Docorg.def())
        .join(JoinType::LeftJoin, entity::docorg::Relation::Docuser.def())
        .filter(entity::docuser::Column::Id.eq(claims.user_id))
        .column_as(entity::docorg::Column::Id, "id")
        .columns([entity::docorg::Column::Raw, entity::docorg::Column::DocuserId, entity::docorg::Column::Status])
        .into_model::<DocorgWithScope>()
        .one(&state.db_conn)
        .await?;

    if res.is_none() {
        return Err(DocumentError::DocumentNotExist.into());
    }
    let res = res.unwrap();

    let convertres = entity::convert::Entity::find_by_id((payload.doc_id, payload.c_type))
        .one(&state.db_conn)
        .await?;
    if convertres.is_none() {
        return Err(DocumentError::DocumentNotConverted.into());
    }
    let convertres = convertres.unwrap();
    match convertres.status {
        0 => return Err(DocumentError::ConvertPending.into()),
        2 => return Err(DocumentError::ConvertFailed.into()),
        _ => {}
    }
    let publish_claims = DocumentClaims {
        iat: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(10)).timestamp(),
        iss: "docuvault".to_owned(),
        doc_id: res.id,
        c_type: payload.c_type,
        scope_id: res.scope_id,
        token_typ: "publish".to_owned(),
    };
    let publish_token = encode(&Header::default(), &publish_claims, &PUBLISH_KEYS.encoding).map_err(|err| DocumentError::from(err))?;
    
    Ok(Json(PublishResponse{
        publish_token,
    }))
}

async fn get_document(State(state): State<AppState>, Json(payload): Json<GetDocumentPayload>) -> Result<impl IntoResponse, GlobalError> {
    let claims = get_claims(payload)?;

    #[derive(FromQueryResult, Serialize, Debug)]
    struct Docs {
        id: i32,
        title: String,
        raw: String,
        data: String,
        status: i32,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
        tag_id: Option<i32>,
        tag_value: Option<String>,
    }
    let res = entity::docorg::Entity::find()
        .filter(entity::docorg::Column::Id.eq(claims.doc_id))
        .join_rev(JoinType::LeftJoin, entity::docorg_tag::Relation::Docorg.def())
        .join_rev(JoinType::LeftJoin, entity::tag::Entity::belongs_to(entity::docorg_tag::Entity)
                  .from(entity::tag::Column::Id)
                  .to(entity::docorg_tag::Column::TagId)
                  .into())
        .join_rev(JoinType::LeftJoin, entity::convert::Relation::Docorg.def())
        .column_as(entity::docorg_tag::Column::TagId, "tag_id")
        .column_as(entity::tag::Column::Value, "tag_value")
        .column_as(entity::convert::Column::Data, "data")
        .into_model::<Docs>()
        .all(&state.db_conn)
        .await?;
    
    if res.len() == 0 {
        return Err(DocumentError::DocumentNotExist.into());
    }
    #[derive(Serialize, Debug)]
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct Tag {
        id: i32,
        value: String,
    }

    #[derive(Serialize, Debug)]
    struct CompDocs {
        id: i32,
        title: String,
        raw: String,
        data: String,
        status: i32,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
        tags: BTreeSet<Tag>,
    }
    let mut ret = CompDocs{
        id: res[0].id,
        title: res[0].title.clone(),
        status: res[0].status,
        raw: res[0].raw.clone(),
        data: res[0].data.clone(),
        created_at: res[0].created_at,
        updated_at: res[0].updated_at,
        tags: BTreeSet::new(),
    };
    for docs in res {
        match docs.tag_id {
            Some(tag_id) => {
                ret.tags.insert(Tag {
                    id: tag_id,
                    value: docs.tag_value.unwrap(),
                });
            }
            None => {
            }
        }
    }
    
    if ret.status != 1 {
        return Err(DocumentError::PrivateDocument.into());
    }
    Ok(Json(ret))
}