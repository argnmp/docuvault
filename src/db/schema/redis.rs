use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;

use macros::redis_schema;
use crate::routes::error::GlobalError;

#[derive(Debug)]
pub struct RedisSchemaHeader {
    pub key: String,
    pub expire_at: Option<usize>,
    pub con: Pool<RedisConnectionManager>,
}
#[redis_schema(scope="scope")]
pub struct Scope{
    pub docuser_id: i32,
    pub name: String,
}
