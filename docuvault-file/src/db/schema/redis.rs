use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;

use macros::redis_schema;

use crate::error::GlobalError;

#[derive(Debug)]
pub struct RedisSchemaHeader {
    pub key: String,
    pub expire_at: Option<usize>,
    pub con: Pool<RedisConnectionManager>,
}
#[redis_schema(scoipe="file")]
pub struct DocFile{
    pub name: String,
    pub ftype: String,
    pub size: u64,
    pub data: Vec<u8>,
}
