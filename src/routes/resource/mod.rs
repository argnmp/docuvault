use std::{sync::Arc, collections::{HashSet, BTreeSet}};

use axum::{routing::post, Router, http::{Method, header}, extract::State, Json, response::IntoResponse};
use sea_orm::FromQueryResult;
use serde::Serialize;
use tower_http::cors::{CorsLayer, Any};
use sea_orm::{entity::*, query::*};

use crate::{AppState, entity, modules::redis::{redis_does_docuser_have_scope}, routes::document::error::DocumentError};

pub mod object;
use object::*;
pub mod error;
use error::*;

use super::error::GlobalError;
use super::auth::object::Claims;
use super::auth::object::Claims as Authenticate;

pub fn create_router(shared_state: AppState) -> Router {
    Router::new()
        .route("/list", post(list))
        .route("/tag", post(tag))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::POST])
                .allow_headers([header::CONTENT_TYPE])
            )
        .with_state(shared_state)
}
async fn tag(State(state): State<AppState>, claims: Claims, Json(payload): Json<TagPayload>) -> Result<impl IntoResponse, GlobalError> {
    /*
     * need for redis optimization
     */

    redis_does_docuser_have_scope(state.clone(), &payload.scope_id[..], claims.user_id).await?;
    let mut scope_id_cond = Condition::any();
    for scope_id in payload.scope_id {
        scope_id_cond = scope_id_cond.add(entity::docorg_scope::Column::ScopeId.eq(scope_id));
    }
    
    #[derive(FromQueryResult, Serialize, Debug)]
    struct Docs {
        id: i32,
        scope_id: i32,
        tag_id: Option<i32>,
        tag_value: Option<String>,
    }
    let res = entity::docorg::Entity::find()
        .filter(entity::docorg::Column::DocuserId.eq(claims.user_id))
        .join_rev(JoinType::LeftJoin, entity::docorg_scope::Relation::Docorg.def())
        .join_rev(JoinType::LeftJoin, entity::docorg_tag::Relation::Docorg.def())
        .join_rev(JoinType::LeftJoin, entity::tag::Entity::belongs_to(entity::docorg_tag::Entity)
                  .from(entity::tag::Column::Id)
                  .to(entity::docorg_tag::Column::TagId)
                  .into())
        .filter(scope_id_cond)
        .column_as(entity::docorg_scope::Column::ScopeId, "scope_id")
        .column_as(entity::docorg_tag::Column::TagId, "tag_id")
        .column_as(entity::tag::Column::Value, "tag_value")
        .into_model::<Docs>()
        .all(&state.db_conn)
        .await?;
    
    #[derive(Serialize)]
    struct Tag {
        id: i32,
        value: String,
    }
    let mut tag_set = BTreeSet::new(); 
    let tag_set = res.into_iter().fold(tag_set, move |mut acc, m|{
        match m.tag_id {
            Some(id) => {acc.insert((id, m.tag_value.unwrap()));},
            None => (),
        }
        acc
    });
    let tag_vec: Vec<Tag> = tag_set.into_iter().map(|(id, value)| Tag {
        id,
        value,
    }).collect();

    Ok(Json(tag_vec))
}
async fn list(State(state): State<AppState>, claims: Claims, Json(payload): Json<ListPayload>) -> Result<impl IntoResponse, GlobalError> {
    
    /*
     * check user has scope ID
     */
    redis_does_docuser_have_scope(state.clone(), &payload.scope_id[..], claims.user_id).await?;
    

    /*
     * return document list
     */

    #[derive(FromQueryResult, Serialize, Debug)]
    struct Docs {
        id: i32,
        scope_id: i32,
        title: String,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
        tag_id: Option<i32>,
    }
    let unit_size = match payload.unit_size {
        Some(value) => {
            if value==0 {
                return Err(ResourceError::UnitSizeZero.into())
            }
            value
        },
        None => 10000, 
    };
    let unit_number = match payload.unit_number {
        Some(value) => value,
        None => 0,
    };
    let mut scope_id_cond = Condition::any();
    for scope_id in payload.scope_id {
        scope_id_cond = scope_id_cond.add(entity::docorg_scope::Column::ScopeId.eq(scope_id));
    }
    let res = entity::docorg::Entity::find()
        .filter(entity::docorg::Column::DocuserId.eq(claims.user_id))
        .join_rev(JoinType::LeftJoin, entity::docorg_scope::Relation::Docorg.def())
        .join_rev(JoinType::LeftJoin, entity::docorg_tag::Relation::Docorg.def())
        .filter(scope_id_cond)
        .column_as(entity::docorg_scope::Column::ScopeId, "scope_id")
        .column_as(entity::docorg_tag::Column::TagId, "tag_id")
        .order_by_desc(entity::docorg::Column::CreatedAt)
        .order_by_desc(entity::docorg::Column::Id)
        .into_model::<Docs>()
        .paginate(&state.db_conn, unit_size);
        // .all(&state.db_conn)
        // .await?;
    let res = res.fetch_page(unit_number).await?; 
    
    #[derive(Serialize, Debug)]
    struct CompDocs {
        id: i32,
        scope_ids: BTreeSet<i32>,
        title: String,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
        tag_ids: BTreeSet<i32>,
    }
    let mut list: Vec<CompDocs> = Vec::new();
    for docs in res {
        match list.last_mut() {
            Some(compdocs) if compdocs.id == docs.id => {
                compdocs.scope_ids.insert(docs.scope_id); 
                if let Some(tag_id) = docs.tag_id {
                    compdocs.tag_ids.insert(tag_id);
                }
            }
            _ => {
                let mut compdocs = CompDocs{
                    id: docs.id,
                    scope_ids: BTreeSet::new(),
                    title: docs.title,
                    created_at: docs.created_at,
                    updated_at: docs.updated_at,
                    tag_ids: BTreeSet::new(), 
                };
                compdocs.scope_ids.insert(docs.scope_id);
                if let Some(tag_id) = docs.tag_id {
                    compdocs.tag_ids.insert(tag_id);
                }
                list.push(compdocs);

            }
        } 
    } 
    if let Some(tag_id) = payload.tag_id {
        list = list.into_iter().filter(|compdocs| compdocs.tag_ids.contains(&tag_id)).collect::<Vec<CompDocs>>();
    }
    Ok(Json(list))
}
