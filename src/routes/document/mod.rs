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
            let res = entity::docorg::Entity::insert(new_doc).exec(txn).await?;
            let new_docorg_scope = entity::docorg_scope::ActiveModel {
                docorg_id: Set(res.last_insert_id),
                scope_id: Set(payload.scope_id),
                ..Default::default()
            };
            let _ = entity::docorg_scope::Entity::insert(new_docorg_scope).exec(txn).await?;
            Ok(())
        })
    }).await?; 

    // let _ = entity::docorg::Entity::insert(new_document).exec(&state.db_conn).await?;
    

    let mut con = state.redis_conn.get().await?;
    let tags: std::collections::BTreeSet<String> = con.zrange("tags", 0, -1).await?;

    let new_tags = payload.tags.into_iter().filter(|tag| !tags.contains(&tag[..])).map(|tag| (0, tag)).collect::<Vec<_>>();

    if new_tags.len() > 0{
        let _:() = con.zadd_multiple("tags", &new_tags[..]).await?;

        // background db update
        // can add function that the last background db work is successful or not
        let db_conn = state.db_conn.clone();
        tokio::spawn(async move {

            let models = new_tags.into_iter().map(|(_, tag)| entity::tag::ActiveModel {
                value: Set(tag),
                ..Default::default()
            });
            entity::tag::Entity::insert_many(models).exec(&db_conn).await?;

            //????????
            Ok::<(), GlobalError>(())
        });
    }

    Ok(())
}
