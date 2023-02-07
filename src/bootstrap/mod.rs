use redis::AsyncCommands;
use sea_orm::{entity::*, query::*};

use crate::{AppState, entity, redis_schema, db::macros::RedisSchemaHeader};

pub async fn bootstrap(state: AppState) {
    let tags = entity::tag::Entity::find()
        .order_by_asc(entity::tag::Column::Value)
        .all(&state.db_conn)
        .await
        .expect("tag loading failes");
    
    let sorted_tags = tags.into_iter().map(|m|(0, m.value)).collect::<Vec<_>>();

    let mut con = state.redis_conn.get().await.unwrap();
    let res:() = con.zadd_multiple("tags", &sorted_tags[..]).await.expect("setting tags failed");
     
}
