use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use axum::{Router, extract::State, Json, response::IntoResponse, routing::post, middleware::from_extractor_with_state};
use redis::AsyncCommands;
use sea_orm::{entity::*, query::*};
use tokio::time::sleep;
use crate::{AppState, entity};

pub mod error;
use error::*;
mod object;
use object::*;


use super::error::GlobalError;
use super::auth::object::Claims;
use super::auth::object::Claims as Authenticate;

pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/create", post(create_document))
        .with_state(shared_state)
}

async fn create_document(State(state): State<AppState>, claims: Claims, Json(payload): Json<CreateDocumentPayload>) -> Result<impl IntoResponse, GlobalError>{
    
    let res = entity::scope::Entity::find()
        .filter(Condition::all()
                .add(entity::scope::Column::DocuserId.eq(claims.user_id))
                .add(entity::scope::Column::Id.eq(payload.scope_id))
                )
        .one(&state.db_conn)
        .await?;

    if res.is_none() {
        return Err(DocumentError::ScopeNotExist.into());
    }
    
    state.db_conn.transaction::<_, (), GlobalError>(|txn|{
        Box::pin(async move {
            let new_doc = entity::docorg::ActiveModel {
                raw: Set(payload.document), 
                docuser_id: Set(claims.user_id),
                status: Set(1),
                ..Default::default()
            };
            let docres = entity::docorg::Entity::insert(new_doc).exec(txn).await?;
            let new_docorg_scope = entity::docorg_scope::ActiveModel {
                docorg_id: Set(docres.last_insert_id),
                scope_id: Set(payload.scope_id),
                ..Default::default()
            };
            let _ = entity::docorg_scope::Entity::insert(new_docorg_scope).exec(txn).await?;


            let mut con = state.redis_conn.get().await?;
            let tags: std::collections::BTreeSet<String> = con.zrange("tags", 0, -1).await?;
            let document_tags = payload.tags.into_iter().collect::<std::collections::BTreeSet<_>>();
            
            let new_tags = document_tags.iter().filter(|tag| !tags.contains(&tag[..])).map(|tag| (0, tag.clone())).collect::<Vec<_>>();


            if new_tags.len() > 0{

                // txn.transaction_with_config::<_, (), GlobalError>(|txn|{
                    // Box::pin(async move {
                        // Ok(())
                    // })
//
                // }, Some(sea_orm::IsolationLevel::Serializable), None).await?;

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
            dbg!(&res);

            let models = res.iter().map(|m|{
                entity::docorg_tag::ActiveModel {
                    docorg_id: Set(docres.last_insert_id),
                    tag_id: Set(m.id),
                    ..Default::default()
                }
            }).collect::<Vec<_>>();
            let res = entity::docorg_tag::Entity::insert_many(models).exec(txn).await?;

            let _:() = con.zadd_multiple("tags", &new_tags[..]).await?;
            Ok(())
        })
    }).await?; 
    


    Ok(())
}
