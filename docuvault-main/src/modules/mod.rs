use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;

use self::tag::TagSetModule;

pub mod redis;
pub mod markdown;
pub mod background;
pub mod grpc;
pub mod tag;

#[derive(Debug)]
pub struct Modules {
    pub tag: TagSetModule,
}
impl Modules {
    pub async fn new(db_conn: DatabaseConnection, redis_conn: Pool<RedisConnectionManager>) -> Self {
        Self {
            tag: TagSetModule::new(db_conn, redis_conn).await
        }
    }
}
