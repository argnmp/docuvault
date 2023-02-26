use std::{sync::Arc, collections::{HashSet, BTreeSet}};

use axum::{routing::post, Router, http::{Method, header, HeaderValue}, extract::State, Json, response::IntoResponse};
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
        .route("/scope/all", post(scope::all))
        .route("/sequence/all", post(sequence::all))
        .route("/sequence/list", post(sequence::list))
        //dashboard only
        .route("/sequence/out", post(sequence::doc_out))
        //dashboard only
        .route("/sequence/in", post(sequence::doc_in))
        //dashboard only
        .route("/sequence/update", post(sequence::update))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::OPTIONS, Method::POST])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .allow_credentials(true)
            )
        .with_state(shared_state)
}
mod scope {
    use super::*;
    pub async fn all(State(state): State<AppState>, claims: Claims) -> Result<impl IntoResponse, GlobalError> {
        let res = entity::scope::Entity::find()
            .filter(entity::scope::Column::DocuserId.eq(claims.user_id))
            .all(&state.db_conn)
            .await?;

        let res = res.into_iter().map(|m|(m.id, m.name)).collect::<Vec<(_,_)>>();

        Ok(Json(ScopeAllResponse{
            scopes: res,
        }))
    }
}
mod sequence {
    use std::collections::HashMap;

    use axum::extract::Path;
    use super::*;
    pub async fn all(State(state): State<AppState>, claims: Claims, Json(payload): Json<SequenceAllPayload>) -> Result<impl IntoResponse, GlobalError> {
        // inquire should be based on scope ids

        redis_does_docuser_have_scope(state.clone(), &payload.scope_ids[..], claims.user_id).await?;
        
        let mut cond = Condition::any();
        for scope_id in payload.scope_ids {
            cond = cond.add(entity::scope_sequence::Column::ScopeId.eq(scope_id));
        }

        #[derive(FromQueryResult, Serialize, Debug)]
        struct Sequence {
            id: i32,
            title: String,
        };
        
        let res = entity::sequence::Entity::find()
            .join_rev(JoinType::LeftJoin, entity::scope_sequence::Relation::Sequence.def())
            .filter(cond)
            .group_by(entity::sequence::Column::Id)
            .into_model::<Sequence>()
            .all(&state.db_conn)
            .await?;

        Ok(Json(res))
    }       

    pub async fn list(State(state): State<AppState>, claims: Claims, Json(payload): Json<SequenceListPayload>) -> Result<impl IntoResponse, GlobalError> {
        //inquire is based on scope ids

        /*
         * check user has scope ID
         */
        redis_does_docuser_have_scope(state.clone(), &payload.scope_ids[..], claims.user_id).await?;

        /*
         * return document list
         */

        let mut scope_id_cond = Condition::any();
        for &scope_id in payload.scope_ids.iter() {
            scope_id_cond = scope_id_cond.add(entity::scope_sequence::Column::ScopeId.eq(scope_id));
        }

        // check if the inquired sequence has scope ids that the user can access(which means the
        // user has scope_ids). 
        let res = entity::scope_sequence::Entity::find()
            .filter(entity::scope_sequence::Column::SequenceId.eq(payload.seq_id))
            .filter(scope_id_cond.clone())
            .one(&state.db_conn)
            .await?;
        if res.is_none() {
            return Err(ResourceError::SequenceNotExist.into());
        }
        
        // can be modified to search without user_id, because it can be extended to many users to
        // work on the same scope
        let mut scope_id_cond = Condition::any();
        for scope_id in payload.scope_ids {
            scope_id_cond = scope_id_cond.add(entity::docorg_scope::Column::ScopeId.eq(scope_id));
        }
        let res = entity::docorg::Entity::find()
            .filter(entity::docorg::Column::DocuserId.eq(claims.user_id))
            .join_rev(JoinType::LeftJoin, entity::docorg_scope::Relation::Docorg.def())
            .join_rev(JoinType::LeftJoin, entity::docorg_tag::Relation::Docorg.def())
            .join_rev(JoinType::LeftJoin, entity::docorg_sequence::Relation::Docorg.def())
            .filter(scope_id_cond)
            .column_as(entity::docorg_scope::Column::ScopeId, "scope_id")
            .column_as(entity::docorg_tag::Column::TagId, "tag_id")
            .column_as(entity::docorg_sequence::Column::SequenceId, "seq_id")
            .column_as(entity::docorg_sequence::Column::Order, "seq_order")
            .filter(entity::docorg_sequence::Column::SequenceId.eq(payload.seq_id))
            .order_by_asc(entity::docorg_sequence::Column::Order)
            .into_model::<SeqDocs>()
            .all(&state.db_conn)
            .await?;

        let mut list: Vec<SeqCompDocs> = Vec::new();
        for docs in res {
            match list.last_mut() {
                Some(compdocs) if compdocs.id == docs.id => {
                    compdocs.scope_ids.insert(docs.scope_id); 
                    if let Some(tag_id) = docs.tag_id {
                        compdocs.tag_ids.insert(tag_id);
                    }
                }
                _ => {
                    let mut compdocs = SeqCompDocs{
                        id: docs.id,
                        seq_id: docs.seq_id,
                        seq_order: docs.seq_order,
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

        Ok(Json(list))
    }       
    pub async fn doc_out(State(state): State<AppState>, claims: Claims, Json(payload): Json<SeqOutPayload>) -> Result<impl IntoResponse, GlobalError> {
        // doesn't matter the scopes thisi document is assigned.
        // only author of document is previleged to do this function
        let res = entity::docorg::Entity::find_by_id(payload.doc_id)
            .filter(entity::docorg::Column::DocuserId.eq(claims.user_id))
            .one(&state.db_conn)
            .await?;
        if res.is_none() {
            return Err(DocumentError::DocumentNotExist.into())
        }

        let res = entity::sequence::Entity::find_by_id(payload.seq_id)
            .filter(entity::sequence::Column::DocuserId.eq(claims.user_id))
            .one(&state.db_conn)
            .await?;
        if res.is_none() {
            return Err(ResourceError::SequenceNotExist.into())
        }
        
        let res = entity::docorg_sequence::Entity::delete_by_id((payload.seq_id, payload.doc_id))
            .exec(&state.db_conn)
            .await?;

        Ok(())
    }       
    pub async fn doc_in(State(state): State<AppState>, claims: Claims, Json(payload): Json<SeqInPayload>) -> Result<impl IntoResponse, GlobalError> {
        let res = entity::docorg::Entity::find_by_id(payload.doc_id)
            .filter(entity::docorg::Column::DocuserId.eq(claims.user_id))
            .one(&state.db_conn)
            .await?;
        if res.is_none() {
            return Err(DocumentError::DocumentNotExist.into())
        }

        let res = entity::sequence::Entity::find_by_id(payload.seq_id)
            .filter(entity::sequence::Column::DocuserId.eq(claims.user_id))
            .one(&state.db_conn)
            .await?;
        if res.is_none() {
            return Err(ResourceError::SequenceNotExist.into())
        }

        state.db_conn.clone().transaction::<_, (), GlobalError>(|txn|{
            Box::pin(async move {

                #[derive(FromQueryResult, Serialize, Debug)]
                struct Seq {
                    last_order: i32,
                };
                let seq = entity::docorg_sequence::Entity::find()
                    .filter(entity::docorg_sequence::Column::SequenceId.eq(payload.seq_id))
                    .select_only()
                    .column_as(entity::docorg_sequence::Column::Order.max(), "last_order")
                    .group_by(entity::docorg_sequence::Column::SequenceId)
                    .into_model::<Seq>()
                    .one(txn)
                    .await?;

                if seq.is_none() {
                    return Err(ResourceError::SequenceNotExist.into());
                }
                let seq = seq.unwrap();

                let sequence = entity::docorg_sequence::ActiveModel {
                    sequence_id: Set(payload.seq_id),
                    docorg_id: Set(payload.doc_id),
                    order: Set(seq.last_order+1),
                };
                sequence.insert(txn).await?;

                Ok(())
            })
        }).await?;
        Ok(())
    }       
    pub async fn update(State(state): State<AppState>, claims: Claims, Json(mut payload): Json<SeqUpdatePayload>) -> Result<impl IntoResponse, GlobalError> {
        // check user has sequence
        let res = entity::sequence::Entity::find_by_id(payload.seq_id)
            .filter(entity::sequence::Column::DocuserId.eq(claims.user_id))
            .one(&state.db_conn)
            .await?;
        if res.is_none() {
            return Err(ResourceError::SequenceNotExist.into())
        }

        payload.order.sort();

        let mut order_map: HashMap<i32, i32> = HashMap::new();
        for (idx, m) in payload.order.into_iter().enumerate() {
            match order_map.insert(m.doc_id, (idx+1) as i32){
                Some(old) => {
                    return Err(ResourceError::SequenceNotSync.into());
                }
                None => {}
            }
        }

        let seq = entity::docorg_sequence::Entity::find()
            .filter(entity::docorg_sequence::Column::SequenceId.eq(payload.seq_id))
            .order_by_asc(entity::docorg_sequence::Column::Order)
            .all(&state.db_conn)
            .await?;

        let mut seq = seq.into_iter().map(|m| m.into_active_model()).collect::<Vec<_>>();
        for m in &mut seq{
            match order_map.get(&m.docorg_id.clone().unwrap()) {
                Some(&value) => m.order = Set(value),
                None => {
                    return Err(ResourceError::SequenceNotSync.into());
                },
            }

        }

        state.db_conn.clone().transaction::<_, (), GlobalError>(|txn|{
            Box::pin(async move {
                for am in seq {
                    am.update(txn).await?;
                }
                Ok(())
            })
        }).await?;

        Ok(())
    }       
    
        
}
async fn tag(State(state): State<AppState>, claims: Claims, Json(payload): Json<TagPayload>) -> Result<impl IntoResponse, GlobalError> {
    /*
     * need for redis optimization
     */

    redis_does_docuser_have_scope(state.clone(), &payload.scope_ids[..], claims.user_id).await?;
    let mut scope_id_cond = Condition::any();
    for scope_id in payload.scope_ids {
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
    redis_does_docuser_have_scope(state.clone(), &payload.scope_ids[..], claims.user_id).await?;
    

    /*
     * return document list
     */

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
    for scope_id in payload.scope_ids {
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
