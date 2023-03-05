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
#[redis_schema(scope="token_pair")]
pub struct TokenPair{
    pub refresh_token: String 
}
#[redis_schema(scope="refresh")]
pub struct Refresh{
    pub ip: String,
}
#[redis_schema(scope="blacklist")]
pub struct BlackList{
    pub status: bool,
}

#[redis_schema(scope="file")]
pub struct File{
    pub name: String,
    pub ftype: String,
    pub size: usize,
    pub data: Vec<u8>,
}
