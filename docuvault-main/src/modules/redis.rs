use std::collections::BTreeSet;

use axum::Json;
use sea_orm::{entity::*, query::*};
use redis::AsyncCommands;
use crate::{AppState, entity, db::schema::redis::{Scope, RedisSchemaHeader}, routes::{error::GlobalError, document::error::DocumentError}};

pub async fn redis_reset_scopes(state: AppState){
    /*
     * delete exising keys
     */
    let mut con = state.redis_conn.get().await.unwrap();
    let existing_keys: Vec<String> = con.keys("scope:*").await.expect("getting existing scope failed");
    if existing_keys.len() > 0 {
        let res: () = con.del(existing_keys).await.expect("deleting existing keys failed");
    }

    /*
     * set scope keys
     */
    let scopes = entity::scope::Entity::find()
        .all(&state.db_conn)
        .await
        .expect("scope loading failed");

    for m in scopes.into_iter() {
        let mut redis_schema = Scope::new(RedisSchemaHeader {
            key: m.id.to_string(),
            expire_at: None,
            con: state.redis_conn.clone(),
        });
        redis_schema.set_docuser_id(m.docuser_id).set_name(m.name);
        redis_schema.flush().await.expect("flush to redis failed");
    }
}

pub async fn redis_does_docuser_have_scope(state: AppState, scope_id: &[i32], docuser_id: i32) -> Result<(), GlobalError>{
    for id in scope_id{
        let redis_header = RedisSchemaHeader {
            key: id.to_string(),
            expire_at: None,
            con: state.redis_conn.clone(),
        }; 
        let mut redis_schema = Scope::new(RedisSchemaHeader {
            key: id.to_string(),
            expire_at: None,
            con: state.redis_conn.clone()
        });
        redis_schema.get_docuser_id().await?;
        match redis_schema.docuser_id {
            Some(id) if id == docuser_id => {},
            _ => {
                return Err(DocumentError::ScopeNotExist.into());
            }
        }
    } 
    Ok(())

}

