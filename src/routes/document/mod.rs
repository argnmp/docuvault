use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use axum::http::{Method, header};
use axum::response::Html;
use axum::routing::{get, options};
use axum::{Router, extract::State, Json, response::IntoResponse, routing::post, middleware::from_extractor_with_state};
use jsonwebtoken::{encode, Header};
use redis::AsyncCommands;
use sea_orm::{entity::*, query::*, FromQueryResult};
use serde::Serialize;
use tokio::time::sleep;
use tower_http::cors::{CorsLayer, Any};
use crate::db::macros::RedisSchemaHeader;
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

pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/test", get(test))
        .route("/create", post(create))
        .route("/publish", post(publish))
        .route("/", post(get_document))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::POST])
                .allow_headers([header::CONTENT_TYPE])
            )
        .with_state(shared_state)
}
async fn create(State(state): State<AppState>, claims: Claims, Json(payload): Json<CreatePayload>) -> Result<impl IntoResponse, GlobalError>{
    
    /*
     * check user has scope
     */
    redis_does_docuser_have_scope(state.clone(), &payload.scope_id[..], claims.user_id).await?;
    
    state.db_conn.transaction::<_, (), GlobalError>(|txn|{
        Box::pin(async move {
            let new_doc = entity::docorg::ActiveModel {
                title: Set(get_title(&payload.document)),
                raw: Set(payload.document), 
                docuser_id: Set(claims.user_id),
                status: Set(1),
                ..Default::default()
            };
            let docres = entity::docorg::Entity::insert(new_doc).exec(txn).await?;

            let scope_ids: Vec<_> = payload.scope_id.iter().map(|&value|{
                entity::docorg_scope::ActiveModel {
                    docorg_id: Set(docres.last_insert_id),
                    scope_id: Set(value),
                    ..Default::default()
                }
            }).collect();

            let _ = entity::docorg_scope::Entity::insert_many(scope_ids).exec(txn).await?;


            let mut con = state.redis_conn.get().await?;
            let tags: std::collections::BTreeSet<String> = con.zrange("tags", 0, -1).await?;
            let document_tags = payload.tags.into_iter().collect::<std::collections::BTreeSet<_>>();
            
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


            //update tags last for the db fail case
            if new_tags.len() > 0{
                let _:() = con.zadd_multiple("tags", &new_tags[..]).await?;
            }
            Ok(())
        })
    }).await?; 
    Ok(())
}

async fn publish(State(state): State<AppState>, claims: Claims, Json(payload): Json<PublishPayload>) -> Result<impl IntoResponse, GlobalError>{

    let res = entity::docorg_scope::Entity::find()
        .filter(
            Condition::all()
                .add(entity::docorg_scope::Column::DocorgId.eq(payload.doc_id))
                .add(entity::docorg_scope::Column::ScopeId.eq(payload.scope_id))
            )
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
    
    let publish_claims = DocumentClaims {
        iat: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(10)).timestamp(),
        iss: "docuvault".to_owned(),
        doc_id: res.id,
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

    let res = entity::docorg::Entity::find()
        .filter(entity::docorg::Column::Id.eq(claims.doc_id))
        .one(&state.db_conn)
        .await?;
    if res.is_none() {
        return Err(DocumentError::DocumentNotExist.into());
    }
    let res = res.unwrap();
    if res.status != 1 {
        return Ok(Json("This is private document".to_string()));
    }
    Ok(Json(res.raw))
}

async fn test(State(state): State<AppState>) -> Result<impl IntoResponse, GlobalError> {
    use macros::redis_schema; 
    #[redis_schema(scope="temp")]
    struct Temp {
        temp1: i32,
        temp2: String,
    }

    let header = RedisSchemaHeader {
            scope: "temp".to_string(), 
            key: "777".to_string(),
            expire_at: None,
            con: state.redis_conn.clone(),
        };
    let mut a = Temp::new(header);
    dbg!(&a);
    a.get_temp1().await?;
    a.get_temp2().await?;
    dbg!(&a);
    a.del_all().await?;
    dbg!(&a);


    Ok(())
}
