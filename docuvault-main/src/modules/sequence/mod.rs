use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;

use self::{application::service::SequenceService, framework::SequencePersistentAdapter};

pub mod domain;
pub mod application;
pub mod framework;
pub mod error;

#[derive(Debug)]
pub struct SequenceModule {
    pub service: SequenceService,
}
impl SequenceModule {
    pub async fn new(db_conn: DatabaseConnection, redis_conn: Pool<RedisConnectionManager>) -> Self {
        Self {
            service: SequenceService::new(
                         Box::new(SequencePersistentAdapter::new(db_conn)),
                         )
        }
    } 
}
