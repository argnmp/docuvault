use redis::AsyncCommands;
use sea_orm::{entity::*, query::*};

use crate::{AppState, entity, modules::redis::redis_reset_scopes};

pub async fn bootstrap(state: AppState) {
    redis_set_tags(state.clone()).await;
    redis_reset_scopes(state.clone()).await;
}

async fn redis_set_tags(state: AppState){
    let tags = entity::tag::Entity::find()
        .order_by_asc(entity::tag::Column::Value)
        .all(&state.db_conn)
        .await
        .expect("tag loading failed");
    
    let tags = tags.into_iter().map(|m|(m.id, m.value)).collect::<Vec<_>>();

    let mut con = state.redis_conn.get().await.unwrap();
    let res:() = con.del("tags").await.expect("deleting existing tags failed");
    if tags.len() > 0{
        let res:() = con.zadd_multiple("tags", &tags[..]).await.expect("setting tags failed");
    }
}

