use once_cell::sync::Lazy;
use sea_orm::DatabaseConnection;

use self::{application::service::TagSetService, framework::TagSetRepositoryAdapter};

pub mod application;
pub mod domain;
pub mod framework;

// orgranize dependencies;
#[derive(Debug)]
pub struct TagSetModule {
    tag_set_service: TagSetService,
}
impl TagSetModule {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self {
            tag_set_service: TagSetService::new(Box::new(TagSetRepositoryAdapter::new(db_conn)))
        }
    }
}
impl TagSetModule {
    pub fn get_service(&self) -> &TagSetService {
        &self.tag_set_service
    }
}
