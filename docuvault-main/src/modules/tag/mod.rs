use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use once_cell::sync::Lazy;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};

use self::{application::service::TagSetService, framework::{TagSetPersistentAdapter, TagSetMemoryAdapter}};
use crate::{AppState, entity, modules::redis::redis_reset_scopes};

pub mod application;
pub mod domain;
pub mod framework;

// orgranize dependencies;
#[derive(Debug)]
pub struct TagSetModule {
    pub service: TagSetService,
}
impl TagSetModule {
    pub async fn new(db_conn: DatabaseConnection, redis_conn: Pool<RedisConnectionManager>) -> Self {
        // initialize module
        let tags = entity::tag::Entity::find()
            .order_by_asc(entity::tag::Column::Value)
            .all(&db_conn)
            .await
            .expect("tag loading failed");

        let tags = tags.into_iter().map(|m|(m.id, m.value)).collect::<Vec<_>>();

        let redis_conn_dup = redis_conn.clone();
        let mut con = redis_conn_dup.get().await.unwrap();
        let res:() = con.del("tags").await.expect("deleting existing tags failed");

        if tags.len() > 0{
            let res:() = con.zadd_multiple("tags", &tags[..]).await.expect("setting tags failed");
        }
            
        Self {
            service: TagSetService::new(
                         Box::new(TagSetPersistentAdapter::new(db_conn)),
                         Box::new(TagSetMemoryAdapter::new(redis_conn)),
                         )
        }
    }
}
